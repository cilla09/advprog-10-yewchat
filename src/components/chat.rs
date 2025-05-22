use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        html! {
            <div class="flex w-screen bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500">
                <div class="flex-none w-56 h-screen bg-white shadow-lg rounded-l-lg">
                    <div class="text-xl p-4 text-center font-bold text-indigo-800">{"Users"}</div>
                    <div class="overflow-y-auto">
                        {
                            self.users.clone().iter().map(|u| {
                                html!{
                                    <div class="flex items-center m-4 p-3 bg-white shadow-md rounded-lg hover:bg-gray-100 transition duration-300">
                                        <div>
                                            <img class="w-14 h-14 rounded-full border-2 border-indigo-500" src={u.avatar.clone()} alt="avatar"/>
                                        </div>
                                        <div class="flex-grow p-3">
                                            <div class="font-semibold text-gray-700">{u.name.clone()}</div>
                                            <div class="text-sm text-gray-400">{"Hey, I'm here!"}</div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>
                <div class="grow h-screen flex flex-col bg-gray-50 rounded-r-lg shadow-xl">
                    <div class="w-full h-14 bg-indigo-600 text-white flex items-center justify-center rounded-t-lg">
                        <div class="text-2xl font-bold">{"💬 Chat Room"}</div>
                    </div>
                    <div class="w-full grow overflow-auto p-4">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html!{
                                    <div class="flex items-end justify-start space-x-4 mb-6">
                                        <img class="w-10 h-10 rounded-full border-2 border-indigo-600" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="max-w-xs p-3 bg-white rounded-xl shadow-md">
                                            <div class="text-sm text-indigo-600 font-semibold">{m.from.clone()}</div>
                                            <div class="text-sm text-gray-600">
                                                {
                                                    if m.message.ends_with(".gif") {
                                                        html!{<img class="mt-2" src={m.message.clone()}/>}
                                                    } else {
                                                        html!{m.message.clone()}
                                                    }
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="w-full h-16 flex items-center px-4 bg-white border-t-2 border-gray-200">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Type your message..." class="block w-full py-2 pl-4 rounded-full bg-gray-200 outline-none text-gray-700 hover:bg-gray-300 transition duration-200"/>
                        <button onclick={submit} class="ml-3 p-3 bg-indigo-600 text-white rounded-full hover:bg-indigo-700 transition duration-300">
                            <img src="https://img.icons8.com/ios-filled/50/ffffff/send.png" alt="send-icon" class="w-6 h-6"/>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}
