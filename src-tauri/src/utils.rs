use futures::stream;

/// 创建基于时间的请求流 - 通用的流创建方法
pub fn create_request_stream(end_time: std::time::Instant) -> impl futures::Stream<Item = usize> {
    let count = 0;
    
    // 使用unfold创建异步流，而不是使用sync迭代器包装，避免卡死
    stream::unfold((count, end_time), |(mut count, end_time)| async move {
        // 检查是否超过测试时间
        if std::time::Instant::now() >= end_time {
            return None;
        }
        
        // 增加计数并返回
        count += 1;
        Some((count, (count, end_time)))
    })
}
