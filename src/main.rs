// 
mod lambda_core;
mod lams;
use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use lambda_core::{beta_reduce, a_equal, Expression};
use lams::{DEMOS, HELLO_TEXT};
//
use std::rc::Rc;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};

fn layout_lambda_text(ui: &egui::Ui, text: &str, is_latest: bool, check_closing_parentheses : bool) -> LayoutJob {
    let mut job = LayoutJob::default();
    let font_id = egui::FontId::monospace(16.0);

    // 当前模式
    let is_dark_mode = ui.visuals().dark_mode;
    // 颜色定义

    //文字
    let color_base = if is_latest {
        if is_dark_mode { egui::Color32::LIGHT_GREEN } else { egui::Color32::from_rgb(0, 100, 0) }
    } else {
        if is_dark_mode { egui::Color32::GRAY } else { egui::Color32::from_rgb(100, 100, 100) }
    };

    // Lambda 关键字
    let color_lambda = if is_dark_mode {
        egui::Color32::from_rgb(200, 100, 255)
    } else {
        egui::Color32::from_rgb(100, 0, 150)
    };

    // 错误色
    let color_error = egui::Color32::RED;

    // 宏名称颜色 ([...])
    let color_macro = if is_dark_mode {
        egui::Color32::from_rgb(100, 200, 255)
    } else {
        egui::Color32::DARK_BLUE
    };

    // 注释色
    let color_comment = if is_dark_mode {
        egui::Color32::from_rgb(100, 200, 100) // 亮绿
    } else {
        egui::Color32::from_rgb(34, 139, 34)   // ForestGreen
    };

    //彩虹括号
    let rainbow_dark = [
        egui::Color32::GOLD,
        egui::Color32::from_rgb(85, 200, 255),
        egui::Color32::from_rgb(255, 160, 100),
    ];
    let rainbow_light = [
        egui::Color32::from_rgb(200, 150, 0),
        egui::Color32::from_rgb(0, 100, 200),
        egui::Color32::from_rgb(200, 80, 0),
    ];
    let rainbow_colors = if is_dark_mode { rainbow_dark } else { rainbow_light };

    // 通过索引访问字符
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    // 预分析错误括号 (跳过注释)
    let mut error_indices = HashSet::new();
    let mut stack = Vec::new();

    if check_closing_parentheses {
        let mut i = 0;
        while i < len {
            // 检查是否是注释开始
            let is_comment_start = if i + 1 < len && chars[i] == '-' && chars[i+1] == '-' {
                true
            } else {
                false
            };
            if is_comment_start {
                // 检查是否是多行注释 --/
                if i + 2 < len && chars[i+2] == '/' {
                    i += 3; // 跳过 --/
                    // 跳过内容直到遇到 /
                    while i < len && chars[i] != '/' {
                        i += 1;
                    }
                    if i < len { i += 1; } // 跳过结束的 /
                } else {
                    // 单行注释 --
                    i += 2; // 跳过 --
                    // 跳过内容直到换行
                    while i < len && chars[i] != '\n' {
                        i += 1;
                    }
                }
                continue; // 进入下一次循环
            }

            // 正常的括号检查
            if chars[i] == '(' {
                stack.push(i);
            } else if chars[i] == ')' {
                if stack.pop().is_none() {
                    error_indices.insert(i);
                }
            }
            i += 1;
        }
        // 剩下未闭合左括号
        for index in stack {
            error_indices.insert(index);
        }
    }


    // LayoutJob
    let mut depth = 0;
    let mut i = 0;

    while i < len {
        let c = chars[i];

        // 先检查注释
        let is_comment_start = if i + 1 < len && c == '-' && chars[i+1] == '-' { true } else { false };

        if is_comment_start {
            let mut comment_content = String::new();

            // 检查多行 --/
            if i + 2 < len && chars[i+2] == '/' {
                // 多行模式
                comment_content.push_str("--/");
                i += 3;
                while i < len {
                    let cc = chars[i];
                    comment_content.push(cc);
                    i += 1;
                    if cc == '/' { break; } // 结束
                }
            } else {
                // 单行模式 --
                comment_content.push_str("--");
                i += 2;
                while i < len {
                    let cc = chars[i];
                    comment_content.push(cc);
                    i += 1;
                    if cc == '\n' { break; } // 结束
                }
            }

            // 渲染整个注释块
            job.append(
                &comment_content,
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: color_comment,
                    ..Default::default()
                }
            );
            continue; // 跳过
        }

        // 正常字符处理
        let mut color = color_base;
        let mut stroke = egui::Stroke::NONE;

        if error_indices.contains(&i) {
            color = color_error;
            stroke = egui::Stroke::new(1.0, color_error);
        } else {
            match c {
                '\\' | 'λ' => {
                    color = color_lambda;
                }
                '[' | ']' => {
                    color = color_macro;
                }
                '(' => {
                    color = rainbow_colors[depth % rainbow_colors.len()];
                    depth += 1;
                }
                ')' => {
                    if depth > 0 {
                        depth -= 1;
                        color = rainbow_colors[depth % rainbow_colors.len()];
                    } else {
                        color = color_error;
                    }
                }
                _ => {}
            }
        }
        // 普通字符
        job.append(
            &c.to_string(),
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color,
                underline: stroke,
                ..Default::default()
            },
        );
        i += 1;
    }
    job
}


//

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "my_font".to_owned(),
        Arc::from(egui::FontData::from_static(include_bytes!("../STKAITI.TTF"))),
    );
    fonts.families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "my_font".to_owned());
    fonts.families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("my_font".to_owned());
    ctx.set_fonts(fonts);
}

// ==========================================
// Egui 应用层
// ==========================================

struct LambdaApp {
    input_text: String,
    history: Vec<Rc<Expression>>,
    env: lambda_core::Environment,
    expanded_indices: HashSet<usize>, //记录哪些行被展开了
    error_msg: Option<String>,
    max_steps: usize,
}

impl Default for LambdaApp {
    fn default() -> Self {
        Self {
            input_text: HELLO_TEXT.to_owned() + "\n\n\n",
            history: Vec::new(),
            env: HashMap::new(),
            expanded_indices: HashSet::new(),
            error_msg: None,
            max_steps: 5000,
        }
    }
}

impl LambdaApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self::default()
    }

    // 解析当前输入并重置历史
    fn parse_input(&mut self) {
        match lambda_core::parse_program(&self.input_text) {
            Ok((user_defined_env, expr)) => {
                self.env = user_defined_env;
                self.history.clear();
                self.history.push(expr);
                self.expanded_indices.clear();
                self.error_msg = None;
            },
            Err(e) => {
                self.error_msg = Some(format!("解析错误: {}", e));
                self.history.clear();
                self.expanded_indices.clear();
            }
        }
    }

    // 执行单步归约
    fn step(&mut self) {
        if let Some(last) = self.history.last() {
            let next = beta_reduce(last, &self.env);
            if !a_equal(last, &next) {
                self.history.push(next);
            }
        } else {
            self.parse_input();
        }
    }

    fn reduce_full(&mut self) {
        self.parse_input();
        if self.history.is_empty() { return; }

        for _ in 0..self.max_steps {
            if let Some(last) = self.history.last() {
                let next = beta_reduce(last, &self.env);
                if a_equal(last, &next) {
                    break;
                }
                self.history.push(next);
            }
        }
    }
}



impl eframe::App for LambdaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        egui::CentralPanel::default().show(ctx, |ui| {
            // 标题栏 
            ui.horizontal(|ui| {
                ui.heading("λ Lambda 求解器");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let is_dark = ui.visuals().dark_mode;
                    let text = if is_dark { "☀ 日间模式" } else { "🌙 夜间模式" };
                    if ui.button(text).clicked() {
                        if is_dark {
                            ctx.set_visuals(egui::Visuals::light());
                        } else {
                            ctx.set_visuals(egui::Visuals::dark());
                        }
                    }
                    ui.menu_button("例子", |ui| {
                        // 设置个最大宽度，防止太窄
                        ui.set_min_width(100.0);
                        for s in DEMOS {
                            // 显示格式: "0  :  (\fx.x)"
                            if ui.button(format!("{}", s[0])).clicked() {
                                // insert_text(s[0]); // 插入宏名，比如 [0]
                                self.input_text = (HELLO_TEXT.to_owned() + s[1]).parse().unwrap();
                                // ui.close_menu();   // 选完关闭菜单
                            }
                        }
                    });
                });
            });


            ui.separator();

            // 如果有全局错误，显示在顶部
            if let Some(err) = &self.error_msg {
                ui.colored_label(egui::Color32::RED, err);
                ui.separator();
            }

            // --- 左右分栏布局 ---
            ui.columns(2, |columns| {
                if let [left_ui, right_ui] = columns {
                    left_ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("单步").clicked() {
                                self.step();
                            }
                            if ui.button("解算").clicked() {
                                self.reduce_full();
                            }
                            /*
                            if ui.button("清空").clicked() {
                                self.input_text.clear();
                                self.history.clear();
                            }
                             */
                        });
                        ui.separator();

                        // 代码输入框
                        egui::ScrollArea::both()
                            .id_source("input_scroll")
                            .show(ui, |ui| {
                                let mut layouter = |ui: &egui::Ui, string: &dyn egui::TextBuffer, _wrap_width: f32| {
                                    let mut job = layout_lambda_text(ui, string.as_str(), false,true);
                                    job.wrap.max_width = f32::INFINITY;
                                    ui.painter().layout_job(job)
                                };

                                ui.add(
                                    egui::TextEdit::multiline(&mut self.input_text)
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY)
                                        .layouter(&mut layouter)
                                        .lock_focus(true)
                                );
                            });
                    });

                    // ============================
                    // 右侧：结果展示区 (使用 right_ui)
                    // ============================
                    right_ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("重置").clicked() {
                                self.parse_input();
                            }
                            // 还可以加一个“全部折叠”按钮
                            if !self.expanded_indices.is_empty() {
                                if ui.button("全部折叠").clicked() {
                                    self.expanded_indices.clear();
                                }
                            }

                        });
                        ui.separator();

                        if self.history.is_empty() {
                            ui.label(egui::RichText::new("雪莉酱卡哇伊呆苏ki~").weak());
                        } else {
                            egui::ScrollArea::both()
                                .stick_to_bottom(true)
                                .show(ui, |ui| {
                                    for (i, expr) in self.history.iter().enumerate() {
                                        // 获取完整字符串
                                        let full_str = format!("{}", expr);
                                        let total_len = full_str.chars().count();
                                        let threshold = 123; // 折叠阈值
                                        let is_expanded = self.expanded_indices.contains(&i);
                                        let is_latest = i == self.history.len() - 1;

                                        ui.horizontal(|ui| {
                                            ui.label(format!("{}.", i));

                                            if total_len <= threshold {
                                                // === 情况 A: 短行，正常显示 ===
                                                let job = layout_lambda_text(ui, &full_str, is_latest, false);
                                                ui.label(job);
                                            } else {
                                                // === 情况 B: 长行 ===
                                                ui.vertical(|ui| {
                                                    // 上半部分：显示文本（截断或完整）
                                                    if is_expanded {
                                                        // 展开状态：显示全部高亮
                                                        // 注意：这在大字符串下依然会卡，但这是用户主动点的
                                                        let job = layout_lambda_text(ui, &full_str, is_latest, false);
                                                        ui.label(job);
                                                    } else {
                                                        // 折叠状态：只截取前100个字符进行高亮，性能飞快
                                                        let sub_str: String = full_str.chars().take(threshold).collect();
                                                        let mut job = layout_lambda_text(ui, &sub_str, is_latest, false);
                                                        // 手动追加一个灰色的 "..."
                                                        job.append(
                                                            " ...",
                                                            0.0,
                                                            TextFormat {
                                                                color: egui::Color32::GRAY,
                                                                font_id: egui::FontId::monospace(16.0),
                                                                ..Default::default()
                                                            }
                                                        );
                                                        ui.label(job);
                                                    }

                                                    // 下半部分：控制栏
                                                    ui.horizontal(|ui| {

                                                        let btn_text = if is_expanded { "收起" } else { "展开" };
                                                        if ui.small_button(btn_text).clicked() {
                                                            if is_expanded {
                                                                self.expanded_indices.remove(&i);
                                                            } else {
                                                                self.expanded_indices.insert(i);
                                                            }
                                                        }
                                                        ui.label(
                                                            egui::RichText::new(format!("(>_<)-该步骤很长: {} 符", total_len))
                                                                //.size(10.0)
                                                                .color(egui::Color32::GRAY)
                                                        );
                                                    });
                                                });
                                            }
                                        });

                                        // 渲染箭头
                                        if i < self.history.len() - 1 {
                                            ui.label(egui::RichText::new("  ↓ β 规约").weak());
                                        }
                                        
                                        ui.separator();
                                    }
                                });
                        }
                    });
                }
            });
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1234.0, 666.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MF Lambda Solver",
        options,
        Box::new(|cc| Ok(Box::new(LambdaApp::new(cc)))),
    )
}