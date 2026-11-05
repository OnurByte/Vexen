//! Vexen Desktop: a calm window onto one checkout's evidence-backed memory.
//!
//! It only reads: the notes, where each came from, and whether the file it describes still
//! matches. Saving and recording stay with the agent and the `vexen` command.
mod model;
mod theme;

use gpui_kit::*;
use model::{Filter, Snapshot};
use std::{borrow::Cow, path::PathBuf};
use theme::*;

fn main() {
    let start = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    application().run(move |cx| {
        init(cx);
        cx.text_system()
            .add_fonts(vec![
                Cow::Borrowed(include_bytes!("../assets/Geist-Regular.otf").as_slice()),
                Cow::Borrowed(include_bytes!("../assets/Geist-SemiBold.otf").as_slice()),
                Cow::Borrowed(include_bytes!("../assets/GeistMono-Regular.otf").as_slice()),
            ])
            .expect("bundled Geist fonts");
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1180.), px(760.)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some("Vexen".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        open_window(options, cx, move |_, cx| {
            cx.new(|_| Memory::new(start.clone()))
        })
        .expect("window");
        cx.activate(true);
    });
}

struct Memory {
    start: PathBuf,
    snapshot: Result<Snapshot, String>,
    selected: usize,
    filter: Filter,
}

impl Memory {
    fn new(start: PathBuf) -> Self {
        let snapshot = Snapshot::load(&start);
        Self {
            start,
            snapshot,
            selected: 0,
            filter: Filter::All,
        }
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        self.snapshot = Snapshot::load(&self.start);
        self.selected = 0;
        cx.notify();
    }

    fn chip(
        &self,
        label: &'static str,
        filter: Filter,
        count: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let on = self.filter == filter;
        div()
            .id(label)
            .px_3()
            .py_1()
            .rounded_full()
            .cursor_pointer()
            .text_size(px(14.))
            .bg(if on { rgb(BLUE) } else { rgb(CARD) })
            .text_color(if on { rgb(BG) } else { rgb(SOFT) })
            .hover(|s| if on { s } else { s.bg(rgb(RAISED)) })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.filter = filter;
                this.selected = 0;
                cx.notify();
            }))
            .child(format!("{label} {count}"))
    }

    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let (repo, trail) = match &self.snapshot {
            Ok(s) => (
                s.root.display().to_string(),
                if s.intact {
                    "Evidence chain intact"
                } else {
                    "Evidence chain does not verify"
                },
            ),
            Err(_) => (self.start.display().to_string(), ""),
        };
        div()
            .flex()
            .flex_none()
            .items_center()
            .justify_between()
            .w_full()
            .px_8()
            .py_5()
            .border_b_1()
            .border_color(rgb(LINE))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(26.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(INK))
                            .child("Vexen"),
                    )
                    .child(
                        div()
                            .text_size(px(14.))
                            .font_family(MONO)
                            .text_color(rgb(DIM))
                            .truncate()
                            .child(repo),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap_4()
                    .ml_4()
                    .child(
                        div()
                            .text_size(px(14.))
                            .text_color(if trail.contains("not") {
                                rgb(SOFT)
                            } else {
                                rgb(DIM)
                            })
                            .child(trail),
                    )
                    .child(
                        div()
                            .id("refresh")
                            .px_4()
                            .py_2()
                            .rounded_lg()
                            .cursor_pointer()
                            .bg(rgb(CARD))
                            .text_color(rgb(INK))
                            .hover(|s| s.bg(rgb(RAISED)))
                            .on_click(cx.listener(|this, _, _, cx| this.reload(cx)))
                            .child("Refresh"),
                    ),
            )
    }

    fn list(&self, snapshot: &Snapshot, cx: &mut Context<Self>) -> impl IntoElement {
        let visible = snapshot.visible(self.filter);
        let (all, applies, quiet) = snapshot.counts();
        let mut rows = div()
            .id("notes")
            .flex()
            .flex_col()
            .gap_2()
            .overflow_y_scroll()
            .flex_1()
            .pb_4();
        for (position, &index) in visible.iter().enumerate() {
            let row = &snapshot.rows[index];
            let on = position == self.selected;
            rows = rows.child(
                div()
                    .id(("note", position))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_4()
                    .rounded_xl()
                    .cursor_pointer()
                    .border_1()
                    .border_color(if on { rgb(BLUE) } else { rgb(CARD) })
                    .bg(rgb(CARD))
                    .hover(|s| s.bg(rgb(RAISED)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.selected = position;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_family(MONO)
                            .text_color(rgb(SOFT))
                            .child(row.record.anchor_path.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(15.))
                            .text_color(rgb(INK))
                            .line_clamp(3)
                            .child(row.record.summary.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(if row.applies { rgb(BLUE) } else { rgb(DIM) })
                            .child(row.verdict()),
                    ),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap_4()
            .w(px(430.))
            .flex_none()
            .h_full()
            .pl_8()
            .pr_4()
            .pt_5()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(self.chip("All", Filter::All, all, cx))
                    .child(self.chip("Applies now", Filter::Applies, applies, cx))
                    .child(self.chip("Quiet", Filter::Quiet, quiet, cx)),
            )
            .child(if visible.is_empty() {
                div()
                    .text_color(rgb(DIM))
                    .child("No relevant evidence found.")
                    .into_any_element()
            } else {
                rows.into_any_element()
            })
    }

    fn detail(&self, snapshot: &Snapshot) -> impl IntoElement {
        let visible = snapshot.visible(self.filter);
        let pane = div()
            .id("detail")
            .flex()
            .flex_col()
            .gap_6()
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_y_scroll()
            .pl_4()
            .pr_8()
            .pt_5()
            .pb_8();
        let Some(&index) = visible.get(self.selected) else {
            return pane.child(
                div()
                    .text_color(rgb(DIM))
                    .child("Choose a note to see where it came from."),
            );
        };
        let row = &snapshot.rows[index];
        let record = &row.record;
        let mut outcomes = div().flex().flex_col().gap_1();
        if record.outcomes.is_empty() {
            outcomes = outcomes.child(
                div()
                    .text_color(rgb(DIM))
                    .child("No test result was saved with this note."),
            );
        }
        for outcome in &record.outcomes {
            let failed = outcome.exit_code != 0;
            outcomes = outcomes.child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        div()
                            .text_color(if failed { rgb(BLUE) } else { rgb(SOFT) })
                            .child(if failed {
                                format!("failed, exit {}", outcome.exit_code)
                            } else {
                                "passed".to_string()
                            }),
                    )
                    .child(
                        div()
                            .font_family(MONO)
                            .text_size(px(14.))
                            .text_color(rgb(INK))
                            .child(outcome.test_id.clone()),
                    ),
            );
        }
        let mut conditions = div().flex().flex_col().gap_1();
        if record.conditions.is_empty() {
            conditions = conditions.child(
                div()
                    .text_color(rgb(DIM))
                    .child("No extra conditions were recorded."),
            );
        }
        for condition in &record.conditions {
            conditions =
                conditions.child(div().text_color(rgb(INK)).child(format!("• {condition}")));
        }
        pane.child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_6()
                .rounded_2xl()
                .border_1()
                .border_color(if row.applies { rgb(BLUE) } else { rgb(CARD) })
                .bg(rgb(CARD))
                .child(div().text_size(px(14.)).font_weight(FontWeight::SEMIBOLD).text_color(if row.applies { rgb(BLUE) } else { rgb(DIM) }).child(if row.applies { "STILL APPLIES" } else { "QUIET" }))
                .child(div().text_size(px(22.)).font_weight(FontWeight::SEMIBOLD).text_color(rgb(INK)).child(record.summary.clone()))
                .child(div().text_color(rgb(SOFT)).child(row.explanation())),
        )
        .child(section("Where it came from", div().flex().flex_col().gap_1().child(fact("File", &record.anchor_path)).child(fact("Commit", record.git_commit.as_deref().map(short).as_deref().unwrap_or("not recorded"))).child(fact("File hash", &short(&record.anchor_sha256))).child(fact("Kind", model::kind_label(record.kind)))))
        .child(section("Test results", outcomes))
        .child(section("When it may no longer apply", div().flex().flex_col().gap_2().child(div().text_color(rgb(SOFT)).child("Vexen shows a note only while the file still matches what was saved, and the note was not replaced.")).child(conditions)))
    }
}

fn short(hash: &str) -> String {
    hash.chars().take(12).collect()
}

fn fact(label: &str, value: &str) -> impl IntoElement {
    div()
        .flex()
        .gap_4()
        .child(
            div()
                .w(px(90.))
                .text_color(rgb(DIM))
                .child(label.to_string()),
        )
        .child(
            div()
                .font_family(MONO)
                .text_size(px(14.))
                .text_color(rgb(INK))
                .child(value.to_string()),
        )
}

fn section(title: &'static str, body: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_size(px(14.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(SOFT))
                .child(title),
        )
        .child(body)
}

impl Render for Memory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let shell = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(BG))
            .text_color(rgb(INK))
            .font_family(SANS)
            .text_size(px(16.))
            .child(self.header(cx));
        match &self.snapshot {
            Ok(s) if s.rows.is_empty() => shell.child(
                div().flex().flex_col().gap_2().p_8().child(div().text_size(px(22.)).font_weight(FontWeight::SEMIBOLD).child("No notes saved yet.")).child(div().text_color(rgb(SOFT)).child("Your agent saves one when a fix fails a test. To see how it works, run `npx @onurbyte/vexen demo` in a terminal.")),
            ),
            Ok(s) => {
                let s = s.clone();
                shell.child(div().flex().flex_1().min_h_0().w_full().overflow_hidden().child(self.list(&s, cx)).child(self.detail(&s)))
            }
            Err(message) => shell.child(
                div().flex().flex_col().gap_2().p_8().child(div().text_size(px(22.)).font_weight(FontWeight::SEMIBOLD).child("This folder has no Vexen memory yet.")).child(div().text_color(rgb(SOFT)).child("Open the app with a repository path, or run `vexen setup` inside the repository first.")).child(div().font_family(MONO).text_size(px(14.)).text_color(rgb(DIM)).child(message.clone())),
            ),
        }
    }
}
