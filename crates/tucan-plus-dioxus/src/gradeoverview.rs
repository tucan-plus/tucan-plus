use dioxus::prelude::*;
use plotters::{
    prelude::*,
    style::text_anchor::{HPos, Pos, VPos},
};
use tucan_types::{
    Tucan,
    gradeoverview::{GradeOverviewRequest, GradeOverviewResponse},
};

use crate::{RcTucanType, common::use_authenticated_data_loader};

#[component]
pub fn GradeOverview(gradeoverview: ReadSignal<GradeOverviewRequest>) -> Element {
    let handler = async |tucan: RcTucanType, current_session, revalidation_strategy, additional| {
        tucan
            .gradeoverview(&current_session, revalidation_strategy, additional)
            .await
    };

    use_authenticated_data_loader(
        handler,
        gradeoverview,
        14 * 24 * 60 * 60,
        60 * 60,
        |gradeoverview: GradeOverviewResponse, reload| {
            rsx! {
                div {
                    h1 {
                        {gradeoverview.module_and_semester}
                        " "
                        button {
                            onclick: reload,
                            r#type: "button",
                            class: "btn btn-secondary",
                            // https://github.com/twbs/icons
                            // The MIT License (MIT)
                            // Copyright (c) 2019-2024 The Bootstrap Authors

                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                width: "16",
                                height: "16",
                                fill: "currentColor",
                                class: "bi bi-arrow-clockwise",
                                view_box: "0 0 16 16",
                                path {
                                    "fill-rule": "evenodd",
                                    d: "M8 3a5 5 0 1 0 4.546 2.914.5.5 0 0 1 .908-.417A6 6 0 1 1 8 2z",
                                }
                                path { d: "M8 4.466V.534a.25.25 0 0 1 .41-.192l2.36 1.966c.12.1.12.284 0 .384L8.41 4.658A.25.25 0 0 1 8 4.466" }
                            }
                        }
                    }
                    h3 { {gradeoverview.modulangebot} }
                    h3 { {gradeoverview.studienleistung} }
                    if let Some(grades) = gradeoverview.maybe_grades {
                        {
                            let mut output = String::new();
                            {
                                let root = SVGBackend::with_string(&mut output, (640, 480))
                                    .into_drawing_area();
                                root.fill(&WHITE)?;
                                let mut chart = ChartBuilder::on(&root)
                                    .x_label_area_size(35)
                                    .y_label_area_size(40)
                                    .margin(5)
                                    .caption("Grade distribution", ("sans-serif", 50.0))
                                    .build_cartesian_2d(
                                        (0usize..grades.columns.len() - 1).into_segmented(),
                                        0usize..grades.columns.iter().max_by_key(|v| v.1).unwrap().1 + 1,
                                    )?;
                                chart
                                    .configure_mesh()
                                    .disable_x_mesh()
                                    .bold_line_style(WHITE.mix(0.3))
                                    .y_desc("Count")
                                    .x_desc("Grade")
                                    .x_label_formatter(
                                        &|i| {
                                            grades
                                                .columns[match i {
                                                    SegmentValue::Exact(_) => unreachable!(),
                                                    SegmentValue::CenterOf(i) => *i,
                                                    SegmentValue::Last => unreachable!(),
                                                }]
                                                .0
                                                .clone()
                                        },
                                    )
                                    .axis_desc_style(("sans-serif", 15))
                                    .draw()?;
                                chart
                                    .draw_series(
                                        Histogram::vertical(&chart)
                                            .style(RED.mix(0.5).filled())
                                            .data(
                                                grades
                                                    .columns
                                                    .iter()
                                                    .enumerate()
                                                    .map(|(idx, column)| { (idx, column.1) }),
                                            ),
                                    )?;
                                chart
                                    .draw_series(
                                        grades
                                            .columns
                                            .iter()
                                            .enumerate()
                                            .map(|(idx, column)| {
                                                Text::new(
                                                    column.1.to_string(),
                                                    (SegmentValue::CenterOf(idx), column.1),
                                                    ("sans-serif", 15)
                                                        .into_text_style(&root)
                                                        .pos(Pos::new(HPos::Center, VPos::Bottom)),
                                                )
                                            }),
                                    )?;
                                root.present()?;
                            }
                            rsx! {
                                div { dangerous_inner_html: output.replace(r#"<svg width="640" height="480""#, "<svg") }
                            }
                        }
                        {
                            grades
                                .infos
                                .iter()
                                .map(|info| {
                                    rsx! {
                                        {info.clone()}
                                        br {}
                                    }
                                })
                        }
                    }
                }
            }
        },
    )
}
