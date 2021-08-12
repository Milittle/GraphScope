use crate::channel_id::ChannelInfo;
use crate::communication::decorator::evented::EventEmitPush;
use crate::communication::decorator::ScopeStreamPush;
use crate::communication::IOResult;
use crate::data::MicroBatch;
use crate::graph::Port;
use crate::progress::{EndSignal, Weight};
use crate::{Data, Tag};

pub struct BroadcastBatchPush<D: Data> {
    pub ch_info: ChannelInfo,
    //pool: MemBatchPool<D>,
    pushes: Vec<EventEmitPush<D>>,
}

impl<D: Data> BroadcastBatchPush<D> {
    pub fn new(ch_info: ChannelInfo, pushes: Vec<EventEmitPush<D>>) -> Self {
        BroadcastBatchPush { ch_info, pushes }
    }
}

impl<T: Data> ScopeStreamPush<MicroBatch<T>> for BroadcastBatchPush<T> {
    fn port(&self) -> Port {
        self.ch_info.source_port
    }

    fn push(&mut self, tag: &Tag, msg: MicroBatch<T>) -> IOResult<()> {
        for i in 1..self.pushes.len() {
            // TODO: avoid clone msg;
            self.pushes[i].push(tag, msg.clone())?;
        }
        self.pushes[0].push(tag, msg)
    }

    fn push_last(&mut self, msg: MicroBatch<T>, mut end: EndSignal) -> IOResult<()> {
        end.update_to(Weight::all());
        for i in 1..self.pushes.len() {
            // TODO: avoid clone msg;
            self.pushes[i].push_last(msg.clone(), end.clone())?;
        }
        self.pushes[0].push_last(msg, end)
    }

    fn notify_end(&mut self, mut end: EndSignal) -> IOResult<()> {
        end.update_to(Weight::all());
        for i in 1..self.pushes.len() {
            self.pushes[i].notify_end(end.clone())?;
        }
        self.pushes[0].notify_end(end)
    }

    fn flush(&mut self) -> IOResult<()> {
        for p in self.pushes.iter_mut() {
            p.flush()?;
        }
        Ok(())
    }

    fn close(&mut self) -> IOResult<()> {
        for p in self.pushes.iter_mut() {
            p.close()?;
        }
        Ok(())
    }
}