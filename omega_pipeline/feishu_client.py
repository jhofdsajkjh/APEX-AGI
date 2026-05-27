"""
OMEGA AGI - Feishu (Lark) Client
飞书/飞书开放平台客户端，支持消息收发、卡片消息、群通知
"""
import json
import time
import logging
from typing import Optional, Dict, Any, List, Callable
from dataclasses import dataclass, field
from enum import Enum

import requests

logger = logging.getLogger(__name__)

# 飞书API基础URL
FEISHU_BASE_URL = "https://open.feishu.cn/open-apis"


class MessageType(Enum):
    """消息类型"""
    TEXT = "text"
    POST = "post"
    IMAGE = "image"
    FILE = "file"
    AUDIO = "audio"
    MEDIA = "media"
    STICKER = "sticker"
    INTERACTIVE = "interactive"
    SHARE_CHAT = "share_chat"
    SHARE_USER = "share_user"


class FeishuClient:
    """飞书客户端"""
    
    def __init__(
        self,
        app_id: str = "",
        app_secret: str = "",
        bot_name: str = "OMEGA AGI Bot"
    ):
        self.app_id = app_id
        self.app_secret = app_secret
        self.bot_name = bot_name
        self._access_token: Optional[str] = None
        self._token_expires_at: float = 0
        self.session = requests.Session()
        
    @classmethod
    def from_env(cls) -> Optional["FeishuClient"]:
        """从环境变量创建客户端"""
        app_id = os.environ.get("FEISHU_APP_ID", "")
        app_secret = os.environ.get("FEISHU_APP_SECRET", "")
        if not app_id or not app_secret:
            logger.warning("FEISHU_APP_ID or FEISHU_APP_SECRET not set")
            return None
        return cls(app_id=app_id, app_secret=app_secret)
    
    def _get_access_token(self) -> Optional[str]:
        """获取tenant_access_token，带缓存"""
        # 检查缓存是否有效（提前5分钟刷新）
        if self._access_token and time.time() < self._token_expires_at - 300:
            return self._access_token
        
        url = f"{FEISHU_BASE_URL}/auth/v3/tenant_access_token/internal"
        payload = {
            "app_id": self.app_id,
            "app_secret": self.app_secret
        }
        
        try:
            resp = self.session.post(url, json=payload, timeout=10)
            data = resp.json()
            
            if data.get("code") == 0:
                self._access_token = data["tenant_access_token"]
                self._token_expires_at = time.time() + data.get("expire", 7200)
                logger.info("Feishu access token refreshed")
                return self._access_token
            else:
                logger.error(f"Failed to get access token: {data}")
                return None
        except Exception as e:
            logger.error(f"Error getting access token: {e}")
            return None
    
    def _headers(self) -> Dict[str, str]:
        """构建认证头"""
        token = self._get_access_token()
        return {
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json"
        }
    
    def send_text(self, receive_id: str, content: str, msg_type: str = "open_id") -> Optional[str]:
        """
        发送文本消息
        
        Args:
            receive_id: 接收者ID (open_id/chat_id)
            content: 消息内容
            msg_type: receive_id的类型 (open_id/chat_id)
        
        Returns:
            message_id if success
        """
        url = f"{FEISHU_BASE_URL}/im/v1/messages"
        params = {"receive_id_type": msg_type}
        
        payload = {
            "receive_id": receive_id,
            "msg_type": MessageType.TEXT.value,
            "content": json.dumps({"text": content})
        }
        
        try:
            resp = self.session.post(
                url,
                params=params,
                headers=self._headers(),
                json=payload,
                timeout=10
            )
            data = resp.json()
            
            if data.get("code") == 0:
                msg_id = data["data"]["message_id"]
                logger.info(f"Message sent: {msg_id}")
                return msg_id
            else:
                logger.error(f"Failed to send message: {data}")
                return None
        except Exception as e:
            logger.error(f"Error sending message: {e}")
            return None
    
    def send_post(
        self,
        receive_id: str,
        title: str,
        content: str,
        msg_type: str = "open_id"
    ) -> Optional[str]:
        """发送富文本消息 (post)"""
        url = f"{FEISHU_BASE_URL}/im/v1/messages"
        params = {"receive_id_type": msg_type}
        
        # 构建post内容
        post_content = [
            [{"tag": "text", "text": title}],
            [{"tag": "text", "text": content}]
        ]
        
        payload = {
            "receive_id": receive_id,
            "msg_type": MessageType.POST.value,
            "content": json.dumps({
                "zh_cn": {
                    "title": title,
                    "content": post_content
                }
            })
        }
        
        try:
            resp = self.session.post(
                url,
                params=params,
                headers=self._headers(),
                json=payload,
                timeout=10
            )
            data = resp.json()
            
            if data.get("code") == 0:
                return data["data"]["message_id"]
            else:
                logger.error(f"Failed to send post: {data}")
                return None
        except Exception as e:
            logger.error(f"Error sending post: {e}")
            return None
    
    def send_card(
        self,
        receive_id: str,
        card: Dict[str, Any],
        msg_type: str = "open_id"
    ) -> Optional[str]:
        """发送卡片消息"""
        url = f"{FEISHU_BASE_URL}/im/v1/messages"
        params = {"receive_id_type": msg_type}
        
        payload = {
            "receive_id": receive_id,
            "msg_type": "interactive",
            "content": json.dumps(card) if isinstance(card, dict) else card
        }
        
        try:
            resp = self.session.post(
                url,
                params=params,
                headers=self._headers(),
                json=payload,
                timeout=10
            )
            data = resp.json()
            
            if data.get("code") == 0:
                return data["data"]["message_id"]
            else:
                logger.error(f"Failed to send card: {data}")
                return None
        except Exception as e:
            logger.error(f"Error sending card: {e}")
            return None
    
    def send_notification(
        self,
        receive_id: str,
        title: str,
        message: str,
        level: str = "info"
    ) -> Optional[str]:
        """
        发送通知消息（使用卡片格式）
        
        Args:
            receive_id: 接收者ID
            title: 通知标题
            message: 通知内容
            level: info/warning/error
        """
        color = {
            "info": "#3498db",
            "warning": "#f39c12",
            "error": "#e74c3c",
            "success": "#2ecc71"
        }.get(level, "#3498db")
        
        card = {
            "config": {"wide_screen_mode": True},
            "elements": [
                {
                    "tag": "div",
                    "text": {
                        "tag": "lark_md",
                        "content": f"**{title}**"
                    }
                },
                {
                    "tag": "div",
                    "text": {
                        "tag": "lark_md",
                        "content": message
                    }
                },
                {"tag": "hr"},
                {
                    "tag": "note",
                    "elements": [
                        {"tag": "text", "text": "OMEGA AGI Supremacy 🔱"}
                    ]
                }
            ]
        }
        
        return self.send_card(receive_id, card)
    
    def reply(self, message_id: str, content: str) -> Optional[str]:
        """回复消息"""
        url = f"{FEISHU_BASE_URL}/im/v1/messages/{message_id}/reply"
        
        payload = {
            "msg_type": MessageType.TEXT.value,
            "content": json.dumps({"text": content})
        }
        
        try:
            resp = self.session.post(
                url,
                headers=self._headers(),
                json=payload,
                timeout=10
            )
            data = resp.json()
            
            if data.get("code") == 0:
                return data["data"]["message_id"]
            else:
                logger.error(f"Failed to reply: {data}")
                return None
        except Exception as e:
            logger.error(f"Error replying: {e}")
            return None
    
    def update_group_notice(self, chat_id: str, content: str) -> bool:
        """更新群公告"""
        url = f"{FEISHU_BASE_URL}/group/v1/chats/{chat_id}/notice"
        
        payload = {"notice": content}
        
        try:
            resp = self.session.patch(
                url,
                headers=self._headers(),
                json=payload,
                timeout=10
            )
            data = resp.json()
            return data.get("code") == 0
        except Exception as e:
            logger.error(f"Error updating group notice: {e}")
            return False
    
    def get_bot_info(self) -> Optional[Dict[str, Any]]:
        """获取机器人信息"""
        url = f"{FEISHU_BASE_URL}/bot/v3/info"
        
        try:
            resp = self.session.get(url, headers=self._headers(), timeout=10)
            data = resp.json()
            
            if data.get("code") == 0:
                return data.get("bot", {})
            else:
                logger.error(f"Failed to get bot info: {data}")
                return None
        except Exception as e:
            logger.error(f"Error getting bot info: {e}")
            return None
    
    def health_check(self) -> bool:
        """健康检查"""
        token = self._get_access_token()
        return token is not None


@dataclass
class FeishuEvent:
    """飞书事件"""
    schema: str = ""
    event_type: str = ""
    event_id: str = ""
    create_time: str = ""
    token: str = ""
    app_id: str = ""
    tenant_key: str = ""
    sender: Optional[Dict[str, Any]] = None
    message: Optional[Dict[str, Any]] = None
    raw: Dict[str, Any] = field(default_factory=dict)
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "FeishuEvent":
        """从字典创建事件"""
        header = data.get("header", {})
        event = data.get("event", {})
        sender = event.get("sender", {})
        message = event.get("message", {})
        
        return cls(
            schema=data.get("schema", ""),
            event_type=header.get("event_type", ""),
            event_id=header.get("event_id", ""),
            create_time=header.get("create_time", ""),
            token=header.get("token", ""),
            app_id=header.get("app_id", ""),
            tenant_key=header.get("tenant_key", ""),
            sender=sender,
            message=message,
            raw=data
        )
    
    def get_text_content(self) -> Optional[str]:
        """从消息事件中提取文本内容"""
        if not self.message:
            return None
        
        msg_type = self.message.get("message_type", "")
        if msg_type != "text":
            return None
        
        try:
            content = json.loads(self.message.get("content", "{}"))
            return content.get("text")
        except (json.JSONDecodeError, TypeError):
            return None
    
    def get_sender_open_id(self) -> Optional[str]:
        """获取发送者open_id"""
        if not self.sender:
            return None
        return self.sender.get("sender_id", {}).get("open_id")
    
    def get_chat_id(self) -> Optional[str]:
        """获取群ID"""
        if not self.message:
            return None
        return self.message.get("chat_id")
    
    def is_group_message(self) -> bool:
        """是否群消息"""
        if not self.message:
            return False
        return self.message.get("chat_type") == "group"


class FeishuBot:
    """飞书Bot - 简化的事件处理"""
    
    def __init__(self, app_id: str, app_secret: str):
        self.client = FeishuClient(app_id, app_secret)
        self._handlers: Dict[str, List[Callable]] = {
            "im.message.receive_v1": [],
        }
        self._default_handlers: List[Callable] = []
    
    @classmethod
    def from_env(cls) -> Optional["FeishuBot"]:
        """从环境变量创建"""
        client = FeishuClient.from_env()
        return cls(client.app_id, client.app_secret) if client else None
    
    def on_message(self, handler: Callable[[FeishuEvent, FeishuClient], None]):
        """注册消息处理器"""
        self._handlers["im.message.receive_v1"].append(handler)
        return handler
    
    def on_default(self, handler: Callable[[FeishuEvent, FeishuClient], None]):
        """注册默认处理器"""
        self._default_handlers.append(handler)
        return handler
    
    def handle_event(self, event_data: Dict[str, Any]) -> bool:
        """处理收到的飞书事件"""
        try:
            event = FeishuEvent.from_dict(event_data)
            event_type = event.event_type
            
            # 查找处理器
            handlers = self._handlers.get(event_type, [])
            handlers.extend(self._default_handlers)
            
            for handler in handlers:
                try:
                    handler(event, self.client)
                except Exception as e:
                    logger.error(f"Handler error: {e}")
            
            return True
        except Exception as e:
            logger.error(f"Error handling event: {e}")
            return False
    
    def send_to_user(self, open_id: str, title: str, message: str, level: str = "info") -> bool:
        """发送通知给用户"""
        msg_id = self.client.send_notification(open_id, title, message, level)
        return msg_id is not None
    
    def send_to_chat(self, chat_id: str, title: str, message: str, level: str = "info") -> bool:
        """发送通知到群"""
        msg_id = self.client.send_notification(chat_id, title, message, level)
        return msg_id is not None


# ============================================================================
# 便捷函数
# ============================================================================

def create_status_card(
    title: str,
    status: str,
    details: Dict[str, Any],
    color: str = "#3498db"
) -> Dict[str, Any]:
    """创建状态卡片"""
    detail_lines = []
    for key, value in details.items():
        detail_lines.append({
            "tag": "div",
            "text": {
                "tag": "lark_md",
                "content": f"• **{key}**: {value}"
            }
        })
    
    return {
        "config": {"wide_screen_mode": True},
        "header": {
            "title": {"tag": "plain_text", "content": title},
            "template": color
        },
        "elements": [
            {
                "tag": "div",
                "text": {
                    "tag": "lark_md",
                    "content": f"**状态**: {status}"
                }
            },
            {"tag": "hr"},
            *detail_lines,
            {"tag": "hr"},
            {
                "tag": "note",
                "elements": [
                    {"tag": "text", "text": f"OMEGA AGI Supremacy 🔱 | {time.strftime('%Y-%m-%d %H:%M:%S')}"}
                ]
            }
        ]
    }


def create_diagnosis_card(
    title: str,
    health_score: float,
    faults: List[Dict[str, Any]],
    fixes: List[str]
) -> Dict[str, Any]:
    """创建诊断报告卡片"""
    health_color = (
        "#2ecc71" if health_score >= 70 else
        "#f39c12" if health_score >= 40 else
        "#e74c3c"
    )
    
    fault_elements = []
    for fault in faults[:5]:  # 最多显示5个
        fault_elements.append({
            "tag": "div",
            "text": {
                "tag": "lark_md",
                "content": f"🔴 **{fault.get('type', 'UNKNOWN')}**: {fault.get('description', '')}"
            }
        })
    
    fix_elements = []
    for fix in fixes[:3]:  # 最多显示3个
        fix_elements.append({
            "tag": "div",
            "text": {
                "tag": "lark_md",
                "content": f"✅ {fix}"
            }
        })
    
    return {
        "config": {"wide_screen_mode": True},
        "header": {
            "title": {"tag": "plain_text", "content": f"🔍 {title}"},
            "template": health_color
        },
        "elements": [
            {
                "tag": "div",
                "text": {
                    "tag": "lark_md",
                    "content": f"**健康评分**: {health_score:.1f}/100"
                }
            },
            {"tag": "hr"},
            {
                "tag": "div",
                "text": {
                    "tag": "lark_md",
                    "content": "**检测到的问题**"
                }
            },
            *fault_elements,
            {"tag": "hr"},
            {
                "tag": "div",
                "text": {
                    "tag": "lark_md",
                    "content": "**建议修复**"
                }
            },
            *fix_elements,
            {"tag": "hr"},
            {
                "tag": "note",
                "elements": [
                    {"tag": "text", "text": f"OMEGA AGI 自诊断 | {time.strftime('%Y-%m-%d %H:%M:%S')}"}
                ]
            }
        ]
    }


if __name__ == "__main__":
    # 测试代码
    import os
    
    logging.basicConfig(level=logging.INFO)
    
    # 从环境变量测试
    client = FeishuClient.from_env()
    if client:
        print("Feishu client created successfully")
        print(f"Bot info: {client.get_bot_info()}")
    else:
        print("Feishu client not configured (missing env vars)")
