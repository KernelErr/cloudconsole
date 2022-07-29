use std::result::Result as stdResult;
use thrussh::server::Session;
use thrussh::*;

pub struct Term<'a> {
    channel_id: ChannelId,
    session: &'a mut Session,
}

impl<'a> Term<'a> {
    pub fn new(channel_id: ChannelId, session: &'a mut Session) -> Self {
        Self {
            channel_id,
            session,
        }
    }
}

impl<'a> std::io::Write for Term<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.session
            .data(self.channel_id, thrussh::CryptoVec::from_slice(buf));
        stdResult::Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        stdResult::Ok(())
    }
}
