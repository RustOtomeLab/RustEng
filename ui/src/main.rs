slint::include_modules!(); // 引入 .slint 文件中的模块

fn main() -> Result<(), slint::PlatformError> {
    let window = MainWindow::new()?;

    // 初始化文本
    window.set_dialogue("你好，欢迎进入 KiteVN。点击继续...".into());

    // 点击事件：切换下一句
    let texts = vec![
        "这是第二句对白。",
        "第三句来了！",
        "这是最后一句了！",
    ];
    let mut current = 0;

    let weak = window.as_weak(); // 为闭包传值准备
    window.on_clicked(move || {
        if let Some(window) = weak.upgrade() {
            if current < texts.len() {
                window.set_dialogue(texts[current].into());
            } else {
                window.set_dialogue("（剧本结束）".into());
            }
            current += 1;
        }
    });

    window.run()
}