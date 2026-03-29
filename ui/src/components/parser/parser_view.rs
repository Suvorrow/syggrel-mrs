use dioxus::prelude::*;
use mrs_exploring_kit::parser::hic_parser::{
    HiCConfig,
    NormalizationMethod,
    extract_contact_matrix
};

#[derive(Props, Clone, PartialEq)]
pub struct ParserToolViewProps {
    pub close_tool: EventHandler<()>,
}

#[component]
pub fn ParserToolView(props: ParserToolViewProps) -> Element {
    let close_tool = props.close_tool;

    // State for UI elements
    let mut input_path = use_signal(String::new);
    let mut output_path = use_signal(String::new);
    let mut chromosome = use_signal(|| String::from("1"));
    let mut resolution = use_signal(|| String::from("1000"));
    let mut normalization_type = use_signal(|| String::from("NONE"));
    let mut start_bin = use_signal(String::new);
    let mut end_bin = use_signal(String::new);

    let mut is_parsing = use_signal(|| false);
    let mut progress = use_signal(|| 0.0);
    let mut status_message = use_signal(String::new);
    let mut log_messages = use_signal(|| vec![String::from("Parser initialized")]);
    let progress_percent = progress() * 100.0;
    let progress_text = format!("Progress: {:.2}%", progress_percent);

    // Handler for the Start Parsing button
    let start_parsing = move |_| {
        if input_path.read().is_empty() {
            let msg = "Error: Please specify the HiC input file path.".to_string();
            status_message.set(msg.clone());
            log_messages.write().push(msg);
            return;
        }

        is_parsing.set(true);
        status_message.set("Initializing parse...".to_string());
        progress.set(0.0);

        log_messages.set(vec![String::from("Starting parse process... ")]);

        // Capture current state values for async task
        let input_path_val = input_path.read().clone();
        let output_path_val = output_path.read().clone();
        let chromosome_val = chromosome.read().clone();
        let resolution_val = resolution.read().clone();
        let norm_type_val = normalization_type.read().clone();
        let start_bin_val = start_bin.read().clone();
        let end_bin_val = end_bin.read().clone();

        // Clone signals for use in async block
        let (mut log_messages_clone, mut status_message_clone, mut is_parsing_clone) = {
            #[allow(clippy::clone_on_copy)]
            (
                log_messages.clone(),
                status_message.clone(),
                is_parsing.clone()
            )
        };

        spawn(async move {
            // Parse configuration values
            let res_parsed: Result<u32, _> = resolution_val.parse();
            let start_bin_parsed: Option<usize> = start_bin_val.parse().ok();
            let end_bin_parsed: Option<usize> = end_bin_val.parse().ok();

            let config = match res_parsed {
                Ok(res) => HiCConfig {
                    normalization: match norm_type_val.as_str() {
                        "VC" => NormalizationMethod::VC,
                        "VCSQRT" => NormalizationMethod::VCSQRT,
                        "KR" => NormalizationMethod::KR,
                        _ => NormalizationMethod::NONE,
                    },
                    resolution: res,
                    chromosome: chromosome_val,
                    start_bin: start_bin_parsed,
                    end_bin: end_bin_parsed,
                },
                Err(e) => {
                    let err_msg = format!("Error parsing resolution: {}", e);
                    log_messages_clone.write().push(err_msg.clone());
                    status_message_clone.set(err_msg);
                    is_parsing_clone.set(false);
                    return;
                }
            };

            log_messages_clone.write().push("Calling hictk via hic_parser...".to_string());
            status_message_clone.set("Calling hictk...".to_string());

            let output_path_opt = if !output_path_val.is_empty() {
                Some(std::path::Path::new(&output_path_val))
            } else {
                None    // ???????
            };

            // Execute blocking parsing operation
            let parse_result = extract_contact_matrix(
                &input_path_val, &config, output_path_opt
            );

            match parse_result {
                Ok(matrix) => {
                    is_parsing_clone.set(false);
                    status_message_clone.set("Parsing completed successfully!".to_string());
                    log_messages_clone.write().push("Parse process finished. Matrix extracted.".to_string());
                    log_messages_clone.write().push(format!("Extracted matrix dimensions: {:?}", matrix.dim()));
                }
                Err(e) => {
                    is_parsing_clone.set(false);
                    let err_msg = format!("Parsing failed: {}", e);
                    status_message_clone.set(err_msg.clone());
                    log_messages_clone.write().push(err_msg);
                }
            }
        });
    };

    rsx! {
        div { class: "parser-overlay",
            div { class: "parser-header",
                h2 { "HiC Parser Tool" }
                button {
                    class: "close-parser-btn",
                    onclick: move |_| close_tool.call(()),
                    "✕"
                }
            }
            div { class: "parser-main-layout",
                div { class: "parser-sidebar",
                    h3 { "Parsing Parameters" }

                    div { class: "control-group",
                        label { "HiC Input File (.hic):" }
                        input {
                            r#type: "text",
                            value: "{input_path}",
                            disabled: is_parsing(),
                            oninput: move |evt| input_path.set(evt.value()),
                        }
                    }

                    div { class: "control-group",
                        label { "Output Directory (if applicable):" }
                        input {
                            r#type: "text",
                            value: "{output_path}",
                            disabled: is_parsing(),
                            oninput: move |evt| output_path.set(evt.value()),
                        }
                    }

                    div { class: "control-group",
                        label { "Chromosome:" }
                        input {
                            r#type: "text",
                            value: "{chromosome}",
                            disabled: is_parsing(),
                            oninput: move |evt| chromosome.set(evt.value()),
                        }
                    }

                    div { class: "control-group",
                        label { "Resolution (bins):" }
                        input {
                            r#type: "text",
                            value: "{resolution}",
                            disabled: is_parsing(),
                            oninput: move |evt| resolution.set(evt.value()),
                        }
                    }

                    div { class: "control-group",
                        label { "Start Bin (optional):" }
                        input {
                            r#type: "text",
                            value: "{start_bin}",
                            disabled: is_parsing(),
                            oninput: move |evt| start_bin.set(evt.value()),
                        }
                    }

                    div { class: "control-group",
                        label { "End Bin (optioanal):" }
                        input {
                            r#type: "text",
                            value: "{end_bin}",
                            disabled: is_parsing(),
                            oninput: move |evt| end_bin.set(evt.value()),
                        }
                    }

                    div { class: "control-group",
                        label { "Normalization:" }
                        select {
                            value: "{normalization_type}",
                            disabled: is_parsing(),
                            oninput: move |evt| normalization_type.set(evt.value()),
                            option { value: "NONE", "None" }
                            option { value: "VC", "Vanilla Coverage (VC)" }
                            option { value: "VCSQRT", "Vanilla Coverage Sqrt (VCSQRT)" }
                            option { value: "KR", "Knight-Ruiz (KR)" }
                        }
                    }

                    button {
                        class: "parse-btn",
                        disabled: is_parsing() || input_path().is_empty(),
                        onclick: start_parsing,
                        "Start HiC Parse"
                    }
                }

                div { class: "parser-results",
                    h3 { "Parsing Results & Progress" }
                    div { class: "status-message", "{status_message}" }

                    div { class: "progress-container",
                        div {
                            class: "progress-bar",
                            width: "{progress() * 100.0}%",
                        }
                    }
                    p { "{progress_text}" }

                    div { class: "parser-logs",
                        h4 { "Log Output" }
                        for log_line in log_messages.read().iter() {
                            p { "{log_line}" }
                        }
                    }
                }
            }
        }
    }
}