//! # Runtime ML Inference
//! Real machine learning inference engine for the OMEGA runtime.
//! Implements a feedforward neural network (Multi-Layer Perceptron) with
//! backpropagation training, model persistence, ensemble support, and
//! feature normalization.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use rand::Rng;

// ============================================================================
// Activation Functions
// ============================================================================

/// Supported activation functions for neural network layers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Activation {
    Relu,
    Sigmoid,
    Tanh,
    Softmax,
    Linear,
}

impl Activation {
    /// Apply the activation function element-wise to a vector.
    pub fn apply(&self, x: &[f32]) -> Vec<f32> {
        match self {
            Activation::Relu => x.iter().map(|&v| if v > 0.0 { v } else { 0.0 }).collect(),
            Activation::Sigmoid => x.iter().map(|&v| 1.0 / (1.0 + (-v).exp())).collect(),
            Activation::Tanh => x.iter().map(|&v| v.tanh()).collect(),
            Activation::Softmax => {
                let max = x.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                let exps: Vec<f32> = x.iter().map(|&v| (v - max).exp()).collect();
                let sum: f32 = exps.iter().sum();
                if sum > 0.0 {
                    exps.iter().map(|&e| e / sum).collect()
                } else {
                    exps
                }
            }
            Activation::Linear => x.to_vec(),
        }
    }

    /// Derivative of the activation function for backpropagation.
    /// For softmax, use the identity derivative (handled via cross-entropy gradient).
    #[allow(unused)]
    pub fn derivative(&self, x: &[f32]) -> Vec<f32> {
        match self {
            Activation::Relu => x.iter().map(|&v| if v > 0.0 { 1.0 } else { 0.0 }).collect(),
            Activation::Sigmoid => {
                let s = self.apply(x);
                s.iter().map(|&v| v * (1.0 - v)).collect()
            }
            Activation::Tanh => {
                let t = self.apply(x);
                t.iter().map(|&v| 1.0 - v * v).collect()
            }
            Activation::Softmax => {
                // Derivative w.r.t. pre-activation: Jacobian is diagonal of softmax*(1-softmax)
                // Simplified: we use cross-entropy gradient directly, so return 1s
                vec![1.0; x.len()]
            }
            Activation::Linear => vec![1.0; x.len()],
        }
    }
}

// ============================================================================
// Matrix Operations
// ============================================================================

/// Matrix multiply: (m x k) @ (k x n) -> (m x n)
pub fn matmul(a: &[Vec<f32>], b: &[Vec<f32>]) -> Vec<Vec<f32>> {
    if a.is_empty() || b.is_empty() || a[0].is_empty() || b[0].is_empty() {
        return Vec::new();
    }
    let m = a.len();
    let k = a[0].len();
    let n = b[0].len();
    // Verify b has k rows
    assert_eq!(b.len(), k, "matmul: inner dimensions must match");
    let mut result = vec![vec![0.0_f32; n]; m];
    for i in 0..m {
        for j in 0..n {
            let mut sum = 0.0;
            for t in 0..k {
                sum += a[i][t] * b[t][j];
            }
            result[i][j] = sum;
        }
    }
    result
}

/// Matrix-vector multiplication: (m x n) @ (n) -> (m)
pub fn matvecmul(a: &[Vec<f32>], b: &[f32]) -> Vec<f32> {
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }
    let m = a.len();
    let n = a[0].len();
    assert_eq!(b.len(), n, "matvecmul: inner dimensions must match");
    let mut result = vec![0.0_f32; m];
    for i in 0..m {
        let mut sum = 0.0;
        for j in 0..n {
            sum += a[i][j] * b[j];
        }
        result[i] = sum;
    }
    result
}

/// Matrix transpose: (m x n) -> (n x m)
pub fn transpose(a: &[Vec<f32>]) -> Vec<Vec<f32>> {
    if a.is_empty() || a[0].is_empty() {
        return Vec::new();
    }
    let m = a.len();
    let n = a[0].len();
    let mut result = vec![vec![0.0_f32; m]; n];
    for i in 0..m {
        for j in 0..n {
            result[j][i] = a[i][j];
        }
    }
    result
}

/// Element-wise vector addition: a + b
pub fn vec_add(a: &[f32], b: &[f32]) -> Vec<f32> {
    assert_eq!(a.len(), b.len(), "vec_add: lengths must match");
    a.iter().zip(b.iter()).map(|(&x, &y)| x + y).collect()
}

/// Scalar-vector multiplication: a * scalar
pub fn vec_scalar_mul(a: &[f32], scalar: f32) -> Vec<f32> {
    a.iter().map(|&x| x * scalar).collect()
}

/// Outer product: (m) x (n) -> (m x n)
pub fn outer(a: &[f32], b: &[f32]) -> Vec<Vec<f32>> {
    let m = a.len();
    let n = b.len();
    let mut result = vec![vec![0.0_f32; n]; m];
    for i in 0..m {
        for j in 0..n {
            result[i][j] = a[i] * b[j];
        }
    }
    result
}

// ============================================================================
// Normalization
// ============================================================================

/// Min-max scaler for feature normalization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinMaxScaler {
    pub min: Vec<f32>,
    pub max: Vec<f32>,
    fitted: bool,
}

impl MinMaxScaler {
    pub fn new() -> Self {
        Self {
            min: Vec::new(),
            max: Vec::new(),
            fitted: false,
        }
    }

    /// Fit the scaler on training data.
    pub fn fit(&mut self, data: &[Vec<f32>]) {
        if data.is_empty() || data[0].is_empty() {
            return;
        }
        let n_features = data[0].len();
        self.min = vec![f32::MAX; n_features];
        self.max = vec![f32::MIN; n_features];
        for row in data {
            for (j, &val) in row.iter().enumerate() {
                if val < self.min[j] {
                    self.min[j] = val;
                }
                if val > self.max[j] {
                    self.max[j] = val;
                }
            }
        }
        self.fitted = true;
    }

    /// Transform data using fitted min/max.
    pub fn transform(&self, data: &[Vec<f32>]) -> Vec<Vec<f32>> {
        if !self.fitted || data.is_empty() {
            return data.to_vec();
        }
        data.iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(j, &val)| {
                        let range = self.max[j] - self.min[j];
                        if range > 1e-12 {
                            (val - self.min[j]) / range
                        } else {
                            0.0
                        }
                    })
                    .collect()
            })
            .collect()
    }

    /// Fit and transform in one step.
    #[allow(unused)]
    pub fn fit_transform(&mut self, data: &[Vec<f32>]) -> Vec<Vec<f32>> {
        self.fit(data);
        self.transform(data)
    }

    /// Inverse transform: map normalized values back to original range.
    #[allow(unused)]
    pub fn inverse_transform(&self, data: &[Vec<f32>]) -> Vec<Vec<f32>> {
        if !self.fitted || data.is_empty() {
            return data.to_vec();
        }
        data.iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .map(|(j, &val)| {
                        let range = self.max[j] - self.min[j];
                        val * range + self.min[j]
                    })
                    .collect()
            })
            .collect()
    }
}

impl Default for MinMaxScaler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Neural Network Core
// ============================================================================

/// A single layer of the neural network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub weights: Vec<Vec<f32>>,
    pub biases: Vec<f32>,
    pub activation: Activation,
}

impl Layer {
    /// Create a new layer with random initialization (Xavier/Glorot).
    pub fn new(input_size: usize, output_size: usize, activation: Activation) -> Self {
        let mut rng = rand::thread_rng();
        let scale = match activation {
            Activation::Relu => (2.0 / input_size as f32).sqrt(),
            _ => (1.0 / input_size as f32).sqrt(),
        };
        let weights: Vec<Vec<f32>> = (0..output_size)
            .map(|_| (0..input_size).map(|_| rng.gen::<f32>() * 2.0 * scale - scale).collect())
            .collect();
        let biases = vec![0.0_f32; output_size];
        Self {
            weights,
            biases,
            activation,
        }
    }

    /// Forward pass through this layer.
    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        let z = vec_add(&matvecmul(&self.weights, input), &self.biases);
        self.activation.apply(&z)
    }
}

/// A feedforward neural network (Multi-Layer Perceptron).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralNetwork {
    pub layers: Vec<Layer>,
    pub learning_rate: f32,
}

impl NeuralNetwork {
    /// Create a new neural network with the specified layer sizes.
    /// `layer_sizes` includes input size and all hidden/output sizes.
    /// Example: [4, 8, 3] -> 4 inputs, 8 hidden, 3 outputs.
    pub fn new(layer_sizes: &[usize], activations: &[Activation], learning_rate: f32) -> Self {
        assert_eq!(
            layer_sizes.len() - 1,
            activations.len(),
            "Must have one activation per hidden+output layer"
        );
        let mut layers = Vec::with_capacity(activations.len());
        for i in 0..activations.len() {
            layers.push(Layer::new(layer_sizes[i], layer_sizes[i + 1], activations[i]));
        }
        Self {
            layers,
            learning_rate,
        }
    }

    /// Create a default network: 1 hidden layer with ReLU, output with Linear.
    #[allow(unused)]
    pub fn new_default(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        Self::new(
            &[input_size, hidden_size, output_size],
            &[Activation::Relu, Activation::Linear],
            0.01,
        )
    }

    /// Forward pass: run input through all layers.
    pub fn predict(&self, input: &[f32]) -> Vec<f32> {
        let mut x = input.to_vec();
        for layer in &self.layers {
            x = layer.forward(&x);
        }
        x
    }

    /// Forward pass that returns intermediate activations (for backprop).
    pub fn forward_with_cache(&self, input: &[f32]) -> (Vec<f32>, Vec<Vec<f32>>) {
        let mut cache: Vec<Vec<f32>> = Vec::with_capacity(self.layers.len() + 1);
        cache.push(input.to_vec());
        let mut x = input.to_vec();
        for layer in &self.layers {
            // Compute z = Wx + b
            let z = vec_add(&matvecmul(&layer.weights, &x), &layer.biases);
            cache.push(z.clone());
            x = layer.activation.apply(&z);
            cache.push(x.clone());
        }
        (x, cache)
    }

    /// Mean Squared Error loss.
    pub fn mse_loss(&self, predicted: &[f32], target: &[f32]) -> f32 {
        assert_eq!(predicted.len(), target.len());
        let sum: f32 = predicted
            .iter()
            .zip(target.iter())
            .map(|(p, t)| (p - t).powi(2))
            .sum();
        sum / predicted.len() as f32
    }

    /// Cross-entropy loss (for classification with softmax output).
    pub fn cross_entropy_loss(&self, predicted: &[f32], target: &[f32]) -> f32 {
        let eps = 1e-15;
        let mut loss = 0.0;
        for (p, t) in predicted.iter().zip(target.iter()) {
            let p_clamped = p.max(eps).min(1.0 - eps);
            loss -= t * p_clamped.ln();
        }
        loss
    }

    /// Compute gradients via backpropagation for one sample.
    pub fn backpropagate(
        &self,
        input: &[f32],
        target: &[f32],
    ) -> Vec<(Vec<Vec<f32>>, Vec<f32>)> {
        // Forward pass with cache
        let n_layers = self.layers.len();
        // Cache: a0, z1, a1, z2, a2, ...
        let (_output, cache) = self.forward_with_cache(input);

        // Compute output layer delta
        let last_a = &cache[cache.len() - 1]; // output activations
        let last_z = &cache[cache.len() - 2]; // output pre-activations

        let delta_last: Vec<f32> = match self.layers[n_layers - 1].activation {
            Activation::Softmax => {
                // For softmax + cross-entropy: delta = predicted - target
                last_a.iter().zip(target.iter()).map(|(p, t)| p - t).collect()
            }
            _ => {
                // MSE: delta = (pred - target) * activation'(z)
                let dz = self.layers[n_layers - 1].activation.derivative(last_z);
                last_a
                    .iter()
                    .zip(target.iter())
                    .zip(dz.iter())
                    .map(|((p, t), d)| (p - t) * d)
                    .collect()
            }
        };

        let mut deltas: Vec<Vec<f32>> = vec![delta_last];

        // Backpropagate through hidden layers
        for l in (0..n_layers - 1).rev() {
            let w_next = &self.layers[l + 1].weights;
            let z_l = &cache[l * 2 + 1]; // pre-activation of layer l
            let dz = self.layers[l].activation.derivative(z_l);

            let w_next_t = transpose(w_next);
            let delta_l = matvecmul(&w_next_t, &deltas[0]);
            let delta_l: Vec<f32> = delta_l.iter().zip(dz.iter()).map(|(d, dz)| d * dz).collect();
            deltas.insert(0, delta_l);
        }

        // Compute weight gradients and bias gradients
        let mut grads = Vec::with_capacity(n_layers);
        for l in 0..n_layers {
            let a_prev = &cache[l * 2]; // input to this layer
            let delta = &deltas[l];
            let w_grad = outer(delta, a_prev); // (output_size x input_size)
            grads.push((w_grad, delta.clone()));
        }

        grads
    }

    /// Update weights using SGD with the computed gradients.
    fn sgd_update(&mut self, grads: &[(Vec<Vec<f32>>, Vec<f32>)]) {
        let lr = self.learning_rate;
        for (l, (w_grad, b_grad)) in grads.iter().enumerate() {
            let layer = &mut self.layers[l];
            let (n_out, n_in) = (layer.weights.len(), layer.weights[0].len());
            for i in 0..n_out {
                for j in 0..n_in {
                    layer.weights[i][j] -= lr * w_grad[i][j];
                }
                layer.biases[i] -= lr * b_grad[i];
            }
        }
    }

    /// Train the network using SGD with full batch.
    pub fn train(
        &mut self,
        inputs: &[Vec<f32>],
        outputs: &[Vec<f32>],
        epochs: usize,
    ) -> Result<()> {
        if inputs.is_empty() || outputs.is_empty() {
            return Err(anyhow!("Training data is empty"));
        }
        if inputs.len() != outputs.len() {
            return Err(anyhow!(
                "Input/output count mismatch: {} vs {}",
                inputs.len(),
                outputs.len()
            ));
        }

        let n_samples = inputs.len();

        for epoch in 0..epochs {
            let mut total_loss = 0.0;

            for i in 0..n_samples {
                let grads = self.backpropagate(&inputs[i], &outputs[i]);
                self.sgd_update(&grads);

                // Compute loss for monitoring
                let pred = self.predict(&inputs[i]);
                let loss = match self.layers.last().map(|l| l.activation) {
                    Some(Activation::Softmax) => self.cross_entropy_loss(&pred, &outputs[i]),
                    _ => self.mse_loss(&pred, &outputs[i]),
                };
                total_loss += loss;
            }

            if epoch % 100 == 0 || epoch == epochs - 1 {
                let avg_loss = total_loss / n_samples as f32;
                tracing::debug!(epoch = %epoch, loss = %avg_loss, "Training epoch");
            }
        }

        Ok(())
    }

    /// Online learning: update model with a single sample (incremental).
    pub fn online_learn(&mut self, input: &[f32], output: &[f32]) -> f32 {
        let grads = self.backpropagate(input, output);
        self.sgd_update(&grads);
        let pred = self.predict(input);
        match self.layers.last().map(|l| l.activation) {
            Some(Activation::Softmax) => self.cross_entropy_loss(&pred, output),
            _ => self.mse_loss(&pred, output),
        }
    }

    /// Save model to a JSON string.
    pub fn save_model(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| anyhow!("Serialization error: {}", e))
    }

    /// Load model from a JSON string.
    pub fn load_model(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| anyhow!("Deserialization error: {}", e))
    }

    /// Compute accuracy: fraction of samples where argmax matches.
    pub fn accuracy(&self, inputs: &[Vec<f32>], targets: &[Vec<f32>]) -> f32 {
        if inputs.is_empty() || targets.is_empty() {
            return 0.0;
        }
        let mut correct = 0;
        for (input, target) in inputs.iter().zip(targets.iter()) {
            let pred = self.predict(input);
            let pred_idx = pred
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0);
            let target_idx = target
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(i, _)| i)
                .unwrap_or(0);
            if pred_idx == target_idx {
                correct += 1;
            }
        }
        correct as f32 / inputs.len() as f32
    }

    /// Compute total loss over a dataset.
    pub fn loss(&self, inputs: &[Vec<f32>], targets: &[Vec<f32>]) -> f32 {
        if inputs.is_empty() {
            return 0.0;
        }
        let mut total = 0.0;
        for (input, target) in inputs.iter().zip(targets.iter()) {
            let pred = self.predict(input);
            total += match self.layers.last().map(|l| l.activation) {
                Some(Activation::Softmax) => self.cross_entropy_loss(&pred, target),
                _ => self.mse_loss(&pred, target),
            };
        }
        total / inputs.len() as f32
    }

    /// Compute precision and recall (binary classification assumed).
    /// target_col: which column in the one-hot / binary output is the positive class.
    pub fn precision_recall(
        &self,
        inputs: &[Vec<f32>],
        targets: &[Vec<f32>],
        positive_threshold: f32,
    ) -> (f32, f32) {
        let mut tp = 0u32;
        let mut fp = 0u32;
        let mut fn_val = 0u32;

        for (input, target) in inputs.iter().zip(targets.iter()) {
            let pred = self.predict(input);
            let pred_class: f32 = if pred[0] >= positive_threshold { 1.0 } else { 0.0 };
            let target_class: f32 = if target[0] >= 0.5 { 1.0 } else { 0.0 };

            if (pred_class - 1.0).abs() < 1e-6f32 && (target_class - 1.0).abs() < 1e-6f32 {
                tp += 1;
            } else if (pred_class - 1.0).abs() < 1e-6f32 && target_class.abs() < 1e-6f32 {
                fp += 1;
            } else if pred_class.abs() < 1e-6f32 && (target_class - 1.0).abs() < 1e-6f32 {
                fn_val += 1;
            }
        }

        let precision = if tp + fp > 0 { tp as f32 / (tp + fp) as f32 } else { 0.0 };
        let recall = if tp + fn_val > 0 { tp as f32 / (tp + fn_val) as f32 } else { 0.0 };

        (precision, recall)
    }
}

impl Default for NeuralNetwork {
    fn default() -> Self {
        Self::new_default(1, 4, 1)
    }
}

// ============================================================================
// Ensemble Support
// ============================================================================

/// Ensemble of neural networks that combines predictions via averaging.
#[derive(Debug, Clone)]
pub struct Ensemble {
    pub models: Vec<NeuralNetwork>,
}

impl Ensemble {
    pub fn new(models: Vec<NeuralNetwork>) -> Self {
        Self { models }
    }

    /// Average predictions from all models.
    pub fn predict(&self, input: &[f32]) -> Vec<f32> {
        if self.models.is_empty() {
            return Vec::new();
        }
        let n_out = self.models[0].predict(input).len();
        let mut sum = vec![0.0_f32; n_out];
        for model in &self.models {
            let pred = model.predict(input);
            for (i, v) in pred.iter().enumerate() {
                sum[i] += v;
            }
        }
        for v in sum.iter_mut() {
            *v /= self.models.len() as f32;
        }
        sum
    }
}

// ============================================================================
// Existing Public Types (kept and extended)
// ============================================================================

/// Configuration for the inference engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub model_cache_size: usize,
    pub default_temperature: f32,
    pub max_batch_size: usize,
    pub timeout_ms: u64,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            model_cache_size: 5,
            default_temperature: 0.7,
            max_batch_size: 32,
            timeout_ms: 30000,
        }
    }
}

/// Handle to a loaded model.
#[derive(Debug, Clone)]
pub struct ModelHandle {
    pub model_id: String,
    pub model_type: String,
    pub loaded_at: chrono::DateTime<chrono::Utc>,
}

/// Result of an inference call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub model_id: String,
    pub output: serde_json::Value,
    pub confidence: Option<f32>,
    pub latency_ms: u64,
}

// ============================================================================
// InferenceEngine — wraps NeuralNetwork for predictions
// ============================================================================

/// ML inference engine with model loading and neural network prediction.
pub struct InferenceEngine {
    config: InferenceConfig,
    models: Arc<RwLock<HashMap<String, ModelHandle>>>,
    networks: Arc<RwLock<HashMap<String, NeuralNetwork>>>,
    scalers: Arc<RwLock<HashMap<String, MinMaxScaler>>>,
    ensembles: Arc<RwLock<HashMap<String, Ensemble>>>,
}

impl InferenceEngine {
    pub fn new(config: InferenceConfig) -> Self {
        Self {
            config,
            models: Arc::new(RwLock::new(HashMap::new())),
            networks: Arc::new(RwLock::new(HashMap::new())),
            scalers: Arc::new(RwLock::new(HashMap::new())),
            ensembles: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(InferenceConfig::default())
    }

    /// Load a model and register it. `model_type` can be "neural_network" or "ensemble".
    /// For neural_network, `model_data` can be a JSON-serialized NeuralNetwork string.
    pub fn load_model(&self, model_id: &str, model_type: &str) -> Result<ModelHandle> {
        let handle = ModelHandle {
            model_id: model_id.to_string(),
            model_type: model_type.to_string(),
            loaded_at: chrono::Utc::now(),
        };

        let count = {
            let mut models = self.models.write();
            models.insert(model_id.to_string(), handle.clone());
            models.len()
        };

        // Create a default neural network for this model if none exists yet
        {
            let mut nets = self.networks.write();
            if !nets.contains_key(model_id) {
                let nn = NeuralNetwork::new_default(
                    self.config.max_batch_size.min(4),
                    8,
                    2,
                );
                nets.insert(model_id.to_string(), nn);
            }
        }

        tracing::info!(model_id = %model_id, model_type = %model_type, "Model loaded ({} cached)", count);
        Ok(handle)
    }

    /// Predict using the neural network associated with the model.
    /// Input is expected as a JSON array of numbers or an object with a "features" array.
    pub fn predict(&self, model_id: &str, input: serde_json::Value) -> Result<InferenceResult> {
        let start = std::time::Instant::now();

        // Check model exists
        let model = self.models.read().get(model_id).cloned()
            .ok_or_else(|| anyhow!("Model '{}' not loaded", model_id))?;

        // Parse input into a Vec<f32>
        let input_vec = json_to_f32_vec(&input)?;

        // Get the neural network and run forward pass
        let result_vec = {
            let nets = self.networks.read();
            let net = nets.get(model_id)
                .ok_or_else(|| anyhow!("No neural network for model '{}'", model_id))?;
            net.predict(&input_vec)
        };

        let confidence = result_vec.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let confidence = if confidence.is_finite() { Some(confidence) } else { None };

        let output = serde_json::json!({
            "prediction": result_vec,
            "model_id": model_id,
            "model_type": model.model_type,
        });

        Ok(InferenceResult {
            model_id: model_id.to_string(),
            output,
            confidence,
            latency_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Train the neural network for a given model.
    pub fn train_model(
        &self,
        model_id: &str,
        inputs: Vec<Vec<f32>>,
        outputs: Vec<Vec<f32>>,
        epochs: usize,
    ) -> Result<()> {
        let mut nets = self.networks.write();
        let net = nets.get_mut(model_id)
            .ok_or_else(|| anyhow!("No neural network for model '{}'", model_id))?;
        net.train(&inputs, &outputs, epochs)
    }

    /// Online learning: update model with one sample.
    pub fn online_learn(&self, model_id: &str, input: &[f32], output: &[f32]) -> Result<f32> {
        let mut nets = self.networks.write();
        let net = nets.get_mut(model_id)
            .ok_or_else(|| anyhow!("No neural network for model '{}'", model_id))?;
        Ok(net.online_learn(input, output))
    }

    /// Save a model's neural network to a JSON string.
    pub fn save_model_to_json(&self, model_id: &str) -> Result<String> {
        let nets = self.networks.read();
        let net = nets.get(model_id)
            .ok_or_else(|| anyhow!("No neural network for model '{}'", model_id))?;
        net.save_model()
    }

    /// Load a neural network from a JSON string and associate it with a model.
    pub fn load_model_from_json(&self, model_id: &str, json: &str) -> Result<()> {
        let net = NeuralNetwork::load_model(json)?;
        let mut nets = self.networks.write();
        nets.insert(model_id.to_string(), net);

        // Ensure the model handle exists
        let mut models = self.models.write();
        if !models.contains_key(model_id) {
            models.insert(
                model_id.to_string(),
                ModelHandle {
                    model_id: model_id.to_string(),
                    model_type: "neural_network".to_string(),
                    loaded_at: chrono::Utc::now(),
                },
            );
        }
        Ok(())
    }

    /// Fit a MinMaxScaler on data for a given model.
    pub fn fit_scaler(&self, model_id: &str, data: &[Vec<f32>]) {
        let mut scalers = self.scalers.write();
        let scaler = scalers.entry(model_id.to_string()).or_insert_with(MinMaxScaler::new);
        scaler.fit(data);
    }

    /// Predict with normalized input (transform via scaler first).
    pub fn predict_normalized(&self, model_id: &str, input: serde_json::Value) -> Result<InferenceResult> {
        let start = std::time::Instant::now();

        let _model = self.models.read().get(model_id).cloned()
            .ok_or_else(|| anyhow!("Model '{}' not loaded", model_id))?;

        let input_vec = json_to_f32_vec(&input)?;

        // Normalize
        let normalized = {
            let scalers = self.scalers.read();
            if let Some(scaler) = scalers.get(model_id) {
                scaler.transform(&[input_vec.clone()]).remove(0)
            } else {
                input_vec.clone()
            }
        };

        let result_vec = {
            let nets = self.networks.read();
            let net = nets.get(model_id)
                .ok_or_else(|| anyhow!("No neural network for model '{}'", model_id))?;
            net.predict(&normalized)
        };

        let confidence = result_vec.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let confidence = if confidence.is_finite() { Some(confidence) } else { None };

        let output = serde_json::json!({
            "prediction": result_vec,
            "model_id": model_id,
            "normalized_input": normalized,
        });

        Ok(InferenceResult {
            model_id: model_id.to_string(),
            output,
            confidence,
            latency_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Add models to an ensemble for combined predictions.
    pub fn create_ensemble(&self, ensemble_id: &str, model_ids: &[String]) -> Result<()> {
        let nets = self.networks.read();
        let mut models_vec = Vec::new();
        for mid in model_ids {
            if let Some(net) = nets.get(mid) {
                models_vec.push(net.clone());
            } else {
                return Err(anyhow!("Model '{}' not found for ensemble", mid));
            }
        }
        let ensemble = Ensemble::new(models_vec);
        let mut ensembles = self.ensembles.write();
        ensembles.insert(ensemble_id.to_string(), ensemble);

        // Register ensemble as a model handle
        let mut models = self.models.write();
        models.insert(
            ensemble_id.to_string(),
            ModelHandle {
                model_id: ensemble_id.to_string(),
                model_type: "ensemble".to_string(),
                loaded_at: chrono::Utc::now(),
            },
        );
        Ok(())
    }

    /// Evaluate model metrics.
    pub fn evaluate(
        &self,
        model_id: &str,
        inputs: &[Vec<f32>],
        targets: &[Vec<f32>],
    ) -> Result<HashMap<String, f32>> {
        let nets = self.networks.read();
        let net = nets.get(model_id)
            .ok_or_else(|| anyhow!("No neural network for model '{}'", model_id))?;

        let mut metrics = HashMap::new();
        metrics.insert("accuracy".to_string(), net.accuracy(inputs, targets));
        metrics.insert("loss".to_string(), net.loss(inputs, targets));
        let (precision, recall) = net.precision_recall(inputs, targets, 0.5);
        metrics.insert("precision".to_string(), precision);
        metrics.insert("recall".to_string(), recall);
        Ok(metrics)
    }

    pub fn unload_model(&self, model_id: &str) -> bool {
        let removed = self.models.write().remove(model_id).is_some();
        self.networks.write().remove(model_id);
        self.scalers.write().remove(model_id);
        self.ensembles.write().remove(model_id);
        removed
    }

    pub fn loaded_models(&self) -> Vec<String> {
        self.models.read().keys().cloned().collect()
    }

    pub fn config(&self) -> &InferenceConfig {
        &self.config
    }
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Parse a serde_json::Value into Vec<f32>.
/// Accepts: JSON array of numbers, or object with "features" array, or number.
fn json_to_f32_vec(value: &serde_json::Value) -> Result<Vec<f32>> {
    match value {
        serde_json::Value::Array(arr) => {
            arr.iter()
                .map(|v| {
                    v.as_f64()
                        .map(|f| f as f32)
                        .ok_or_else(|| anyhow!("Expected numeric value in array, got: {}", v))
                })
                .collect()
        }
        serde_json::Value::Object(map) => {
            if let Some(features) = map.get("features").and_then(|v| v.as_array()) {
                features
                    .iter()
                    .map(|v| {
                        v.as_f64()
                            .map(|f| f as f32)
                            .ok_or_else(|| anyhow!("Expected numeric feature, got: {}", v))
                    })
                    .collect()
            } else {
                Err(anyhow!("Expected JSON array or object with 'features' field"))
            }
        }
        serde_json::Value::Number(n) => {
            n.as_f64().map(|f| vec![f as f32])
                .ok_or_else(|| anyhow!("Invalid number: {}", n))
        }
        _ => Err(anyhow!("Expected JSON array or object, got: {}", value)),
    }
}
