use std::sync::Arc;
use webrtc::data_channel::RTCDataChannel;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use crate::utils;

pub async fn setup_data_channel_handlers(data_channel: Arc<RTCDataChannel>) {
    data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
        let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
        println!("Message from remote: '{}'", msg_str);
        Box::pin(async {})
    }));

    utils::set_data_channel(Arc::clone(&data_channel)).await;
}