use super::super::*;

// size related functions

pub fn frame_size(frame: &MessageFrame) -> usize {
    match *frame {
        MessageFrame::Treq(ref f) => treq_size(f),
        MessageFrame::Rreq(ref f) => 1 + rmsg_size(f),
        MessageFrame::Tdispatch(ref f) => tdispatch_size(f),
        MessageFrame::Rdispatch(ref f) => rdispatch_size(f),
        MessageFrame::Tinit(ref f) => init_size(f),
        MessageFrame::Rinit(ref f) => init_size(f),
        MessageFrame::Tdrain => 0,
        MessageFrame::Rdrain => 0,
        MessageFrame::Tping => 0,
        MessageFrame::Rping => 0,
        MessageFrame::Tlease(_) => 9,
        MessageFrame::Tdiscarded(ref m) => 3 + m.msg.as_bytes().len(),
        MessageFrame::Rerr(ref msg) => msg.as_bytes().len(),
    }
}

pub fn tdispatch_size(msg: &Tdispatch) -> usize {
    let mut size = 2 + // dest size
                   context_size(&msg.contexts) +
                   dtab_size(&msg.dtab);

    size += msg.dest.as_bytes().len();
    size += msg.body.len();
    size
}

pub fn rdispatch_size(msg: &Rdispatch) -> usize {
    1 + context_size(&msg.contexts) + match msg.msg {
        Rmsg::Ok(ref body) => body.len(),
        Rmsg::Error(ref msg) => msg.as_bytes().len(),
        Rmsg::Nack(ref msg) => msg.as_bytes().len(),
    }
}

pub fn treq_size(treq: &Treq) -> usize {
    let mut size = 1; // header count
    for &(_, ref v) in &treq.headers {
        size += 2; // key and value lengths
        size += v.len();
    }

    size + treq.body.len()
}

#[inline]
pub fn rmsg_size(msg: &Rmsg) -> usize {
    match *msg {
        Rmsg::Ok(ref b) => b.len(),
        Rmsg::Error(ref m) => m.as_bytes().len(),
        Rmsg::Nack(ref m) => m.as_bytes().len(),
    }
}

#[inline]
pub fn init_size(init: &Init) -> usize {
    let mut size = 2; // version

    for &(ref k, ref v) in &init.headers {
        // each value preceeded by its len (i32)
        size += 8 + k.len() + v.len();
    }
    size
}

#[inline]
pub fn context_size(contexts: &Contexts) -> usize {
    let mut size = 2; // context size

    for &(ref k, ref v) in contexts {
        size += 4; // two lengths
        size += k.len();
        size += v.len();
    }
    size
}

#[inline]
pub fn dtab_size(table: &Dtab) -> usize {
    let mut size = 2; // context size

    for dentry in &table.entries {
        size += 4; // the two lengths
        size += dentry.key.as_bytes().len();
        size += dentry.val.as_bytes().len();
    }

    size
}
