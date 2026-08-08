use sha2::{Digest, Sha256};

use super::{
    error::{Phase09Error, Phase09Result},
    models::{CanonicalDocumentPayload, TemplateConfiguration},
};

const ALLOWED_SECTIONS: &[&str] = &["REFERENCES", "NOTES", "TOTALS"];
const FORBIDDEN_FRAGMENTS: &[&str] = &[
    "<script",
    "</script",
    "<iframe",
    "<object",
    "<embed",
    "javascript:",
    concat!("http", "://"),
    concat!("https", "://"),
    "file://",
    "localstorage",
    "sessionstorage",
    "@import",
    "url(",
    "data:image/svg",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedSnapshotContent {
    pub canonical_payload_json: String,
    pub html: String,
    pub css: String,
    pub content_sha256: String,
}

pub fn validate_template_configuration(configuration: &TemplateConfiguration) -> Phase09Result<()> {
    for (field, value) in [
        ("documentTitleAr", configuration.document_title_ar.as_str()),
        ("documentTitleFr", configuration.document_title_fr.as_str()),
        ("footerTextAr", configuration.footer_text_ar.as_str()),
        ("footerTextFr", configuration.footer_text_fr.as_str()),
    ] {
        let trimmed = value.trim();
        if field.starts_with("documentTitle") && trimmed.is_empty() {
            return Err(Phase09Error::validation(&format!("{field} is required.")));
        }
        if trimmed.len() > 500 {
            return Err(Phase09Error::validation(&format!("{field} is too long.")));
        }
        validate_controlled_text(trimmed, field)?;
    }

    if !matches!(configuration.spacing.as_str(), "NORMAL" | "COMPACT") {
        return Err(Phase09Error::validation(
            "Template spacing must be NORMAL or COMPACT.",
        ));
    }
    if !matches!(configuration.orientation.as_str(), "PORTRAIT" | "LANDSCAPE") {
        return Err(Phase09Error::validation(
            "Template orientation must be PORTRAIT or LANDSCAPE.",
        ));
    }
    for section in &configuration.enabled_sections {
        if !ALLOWED_SECTIONS.contains(&section.as_str()) {
            return Err(Phase09Error::validation(
                "The template contains an unsupported optional section.",
            ));
        }
    }
    Ok(())
}

fn validate_controlled_text(value: &str, field: &str) -> Phase09Result<()> {
    let normalized = value.to_ascii_lowercase();
    if FORBIDDEN_FRAGMENTS
        .iter()
        .any(|fragment| normalized.contains(fragment))
    {
        return Err(Phase09Error::validation(&format!(
            "{field} contains forbidden executable or remote content."
        )));
    }
    let bytes = normalized.as_bytes();
    for index in 0..bytes.len().saturating_sub(2) {
        if bytes[index] == b'o' && bytes[index + 1] == b'n' {
            let tail = &normalized[index + 2..];
            if let Some(eq) = tail.find('=') {
                let candidate = &tail[..eq];
                if !candidate.is_empty()
                    && candidate.len() <= 32
                    && candidate
                        .chars()
                        .all(|character| character.is_ascii_alphabetic())
                {
                    return Err(Phase09Error::validation(&format!(
                        "{field} contains a forbidden inline event attribute."
                    )));
                }
            }
        }
    }
    if value
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return Err(Phase09Error::validation(&format!(
            "{field} contains unsupported control characters."
        )));
    }
    Ok(())
}

pub fn render_document(
    locale: &str,
    configuration: &TemplateConfiguration,
    payload: &CanonicalDocumentPayload,
) -> Phase09Result<RenderedSnapshotContent> {
    validate_template_configuration(configuration)?;
    if !matches!(locale, "ar-DZ" | "fr-DZ") {
        return Err(Phase09Error::validation("Unsupported locale."));
    }
    let canonical_payload_json =
        serde_json::to_string(payload).map_err(|_| Phase09Error::internal())?;
    let direction = if locale == "ar-DZ" { "rtl" } else { "ltr" };
    let title = if locale == "ar-DZ" {
        &configuration.document_title_ar
    } else {
        &configuration.document_title_fr
    };
    let footer = if locale == "ar-DZ" {
        &configuration.footer_text_ar
    } else {
        &configuration.footer_text_fr
    };
    let company_name = if locale == "ar-DZ" {
        &payload.company_name
    } else if payload.company_legal_name.trim().is_empty() {
        &payload.company_name
    } else {
        &payload.company_legal_name
    };
    let partner_label = if locale == "ar-DZ" {
        "الطرف"
    } else {
        "Partenaire"
    };
    let date_label = if locale == "ar-DZ" {
        "التاريخ"
    } else {
        "Date"
    };
    let due_label = if locale == "ar-DZ" {
        "تاريخ الاستحقاق"
    } else {
        "Échéance"
    };
    let description_label = if locale == "ar-DZ" {
        "البيان"
    } else {
        "Désignation"
    };
    let quantity_label = if locale == "ar-DZ" {
        "الكمية"
    } else {
        "Quantité"
    };
    let unit_price_label = if locale == "ar-DZ" {
        "سعر الوحدة"
    } else {
        "Prix unitaire"
    };
    let ht_label = "HT";
    let tva_label = "TVA";
    let ttc_label = "TTC";

    let mut rows = String::new();
    for line in &payload.lines {
        rows.push_str(&format!(
            "<tr><td class=\"num\">{}</td><td><span class=\"code\">{}</span><span>{}</span></td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td><td class=\"num\">{}</td></tr>",
            line.line_number,
            escape_html(&line.product_code),
            escape_html(&line.description),
            format_quantity(line.quantity_scaled),
            format_scaled(line.unit_price_scaled, 4),
            format_money(line.ht_minor),
            format_money(line.tax_minor),
            format_money(line.ttc_minor),
        ));
    }

    let partner = payload
        .partner_name
        .as_deref()
        .map(escape_html)
        .unwrap_or_else(|| "—".to_owned());
    let partner_address = if configuration.show_partner_address {
        payload
            .partner_address
            .as_deref()
            .map(|address| format!("<p>{}</p>", escape_html(address)))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let company_details = if configuration.show_company_identity {
        let mut details = String::new();
        if let Some(address) = &payload.company_address {
            details.push_str(&format!("<p>{}</p>", escape_html(address)));
        }
        if configuration.show_trade_register {
            if let Some(value) = &payload.company_trade_register {
                details.push_str(&format!("<p>RC: {}</p>", escape_html(value)));
            }
        }
        if configuration.show_tax_identifier {
            if let Some(value) = &payload.company_tax_identifier {
                details.push_str(&format!("<p>NIF: {}</p>", escape_html(value)));
            }
        }
        details
    } else {
        String::new()
    };
    let references = if configuration
        .enabled_sections
        .iter()
        .any(|s| s == "REFERENCES")
        && !payload.references.is_empty()
    {
        let items = payload
            .references
            .iter()
            .map(|value| format!("<li>{}</li>", escape_html(value)))
            .collect::<String>();
        format!("<section class=\"references\"><ul>{items}</ul></section>")
    } else {
        String::new()
    };
    let notes = if configuration.enabled_sections.iter().any(|s| s == "NOTES") {
        payload
            .notes
            .as_deref()
            .map(|value| {
                format!(
                    "<section class=\"notes\"><p>{}</p></section>",
                    escape_html(value)
                )
            })
            .unwrap_or_default()
    } else {
        String::new()
    };
    let payment = if configuration.show_payment_information {
        payload
            .payment_information
            .as_deref()
            .map(|value| format!("<p class=\"payment\">{}</p>", escape_html(value)))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let due = payload
        .due_date
        .as_deref()
        .map(|value| {
            format!(
                "<div><span>{due_label}</span><strong>{}</strong></div>",
                escape_html(value)
            )
        })
        .unwrap_or_default();

    let html = format!(
        "<!doctype html><html lang=\"{locale}\" dir=\"{direction}\"><head><meta charset=\"utf-8\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; img-src data:; style-src 'unsafe-inline'; font-src 'self'; connect-src 'none'; script-src 'none'; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'\"><meta name=\"referrer\" content=\"no-referrer\"><title>{}</title></head><body><main class=\"document\"><header><section class=\"company\"><h1>{}</h1>{}</section><section class=\"identity\"><h2>{}</h2><p class=\"number\">{}</p><p>{}</p></section></header><section class=\"meta\"><div><span>{date_label}</span><strong>{}</strong></div>{due}<div><span>{partner_label}</span><strong>{partner}</strong>{partner_address}</div></section><table><thead><tr><th>#</th><th>{description_label}</th><th>{quantity_label}</th><th>{unit_price_label}</th><th>{ht_label}</th><th>{tva_label}</th><th>{ttc_label}</th></tr></thead><tbody>{rows}</tbody></table><section class=\"totals\"><div><span>HT</span><strong>{}</strong></div><div><span>TVA</span><strong>{}</strong></div><div class=\"grand\"><span>TTC</span><strong>{} {}</strong></div></section>{payment}{references}{notes}<footer>{}</footer></main></body></html>",
        escape_html(title),
        escape_html(company_name),
        company_details,
        escape_html(title),
        escape_html(&payload.document_number),
        escape_html(&payload.document_status),
        escape_html(&payload.commercial_date),
        format_money(payload.total_ht_minor),
        format_money(payload.total_tax_minor),
        format_money(payload.total_ttc_minor),
        escape_html(&payload.currency_code),
        escape_html(footer),
    );

    let compact = configuration.spacing == "COMPACT";
    let landscape = configuration.orientation == "LANDSCAPE";
    let css = format!(
        "@page{{size:A4 {};margin:{}mm}}*{{box-sizing:border-box}}html{{font-family:'Noto Sans Arabic','Segoe UI',sans-serif;color:#111827;background:#fff}}body{{margin:0;direction:{direction}}}.document{{width:100%;font-size:{}px;line-height:1.45}}header{{display:flex;justify-content:space-between;gap:20px;border-bottom:2px solid #111827;padding-bottom:12px}}h1,h2,p{{margin:0}}h1{{font-size:22px}}h2{{font-size:26px}}.identity{{text-align:end}}.number{{font-weight:700;font-size:18px}}.company p,.meta p{{color:#374151}}.meta{{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:12px;margin:16px 0}}.meta>div{{border:1px solid #d1d5db;padding:10px;min-height:64px}}.meta span{{display:block;color:#6b7280;font-size:11px}}table{{width:100%;border-collapse:collapse;table-layout:fixed}}thead{{display:table-header-group}}tr{{break-inside:avoid}}th,td{{border:1px solid #d1d5db;padding:{}px;vertical-align:top;overflow-wrap:anywhere}}th{{background:#f3f4f6;font-weight:700}}th:nth-child(1),td:nth-child(1){{width:5%}}th:nth-child(2),td:nth-child(2){{width:35%}}th:nth-child(n+3),td:nth-child(n+3){{width:12%}}.code{{display:block;font-size:10px;color:#6b7280}}.num{{font-variant-numeric:tabular-nums;text-align:end;direction:ltr}}.totals{{margin:14px 0 0 auto;width:min(360px,100%);break-inside:avoid}}.totals div{{display:flex;justify-content:space-between;border-bottom:1px solid #d1d5db;padding:6px}}.totals .grand{{font-size:18px;border:2px solid #111827}}.references,.notes,.payment{{margin-top:14px;break-inside:avoid}}footer{{margin-top:20px;padding-top:10px;border-top:1px solid #d1d5db;text-align:center;color:#4b5563}}@media print{{html,body{{print-color-adjust:exact;-webkit-print-color-adjust:exact}}}}",
        if landscape { "landscape" } else { "portrait" },
        if compact { 8 } else { 12 },
        if compact { 10 } else { 12 },
        if compact { 5 } else { 8 },
    );

    reject_rendered_forbidden(&html, &css)?;
    let mut hasher = Sha256::new();
    hasher.update(canonical_payload_json.as_bytes());
    hasher.update([0]);
    hasher.update(html.as_bytes());
    hasher.update([0]);
    hasher.update(css.as_bytes());
    let content_sha256 = format!("{:x}", hasher.finalize());
    Ok(RenderedSnapshotContent {
        canonical_payload_json,
        html,
        css,
        content_sha256,
    })
}

fn reject_rendered_forbidden(html: &str, css: &str) -> Phase09Result<()> {
    let combined = format!("{html}\n{css}").to_ascii_lowercase();
    for fragment in FORBIDDEN_FRAGMENTS {
        if combined.contains(fragment) {
            return Err(Phase09Error::validation(
                "Rendered output contains forbidden executable or remote content.",
            ));
        }
    }
    for tag in ["<iframe", "<object", "<embed", "<script"] {
        if combined.contains(tag) {
            return Err(Phase09Error::validation(
                "Rendered output contains a forbidden element.",
            ));
        }
    }
    Ok(())
}

pub fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

pub fn format_money(value_minor: i64) -> String {
    format_scaled(value_minor, 2)
}

pub fn format_quantity(value_scaled: i64) -> String {
    trim_scaled(format_scaled(value_scaled, 6))
}

pub fn format_rate(value_scaled: i64) -> String {
    trim_scaled(format_scaled(value_scaled, 4))
}

pub fn format_scaled(value: i64, scale: u32) -> String {
    let negative = value < 0;
    let absolute = i128::from(value).abs();
    let divisor = 10_i128.pow(scale);
    let whole = absolute / divisor;
    let fraction = absolute % divisor;
    if scale == 0 {
        format!("{}{whole}", if negative { "-" } else { "" })
    } else {
        format!(
            "{}{whole}.{fraction:0width$}",
            if negative { "-" } else { "" },
            width = scale as usize
        )
    }
}

fn trim_scaled(mut value: String) -> String {
    if value.contains('.') {
        while value.ends_with('0') {
            value.pop();
        }
        if value.ends_with('.') {
            value.pop();
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configuration() -> TemplateConfiguration {
        TemplateConfiguration {
            document_title_ar: "فاتورة".into(),
            document_title_fr: "Facture".into(),
            show_logo: true,
            show_company_identity: true,
            show_trade_register: true,
            show_tax_identifier: true,
            show_partner_address: true,
            show_payment_information: true,
            footer_text_ar: "شكراً".into(),
            footer_text_fr: "Merci".into(),
            spacing: "NORMAL".into(),
            orientation: "PORTRAIT".into(),
            enabled_sections: vec!["REFERENCES".into(), "NOTES".into(), "TOTALS".into()],
        }
    }

    #[test]
    fn rejects_executable_and_remote_template_content() {
        for unsafe_value in [
            "<script>alert(1)</script>".to_owned(),
            format!("{}{}", "https", "://example.test/logo.png"),
            "onclick=evil()".to_owned(),
            "javascript:alert(1)".to_owned(),
        ] {
            let mut candidate = configuration();
            candidate.footer_text_fr = unsafe_value.clone();
            assert!(
                validate_template_configuration(&candidate).is_err(),
                "{unsafe_value}"
            );
        }
    }

    #[test]
    fn fixed_point_formatting_is_deterministic() {
        assert_eq!(format_money(12345), "123.45");
        assert_eq!(format_quantity(1_500_000), "1.5");
        assert_eq!(format_scaled(12345, 4), "1.2345");
        assert_eq!(format_rate(190000), "19");
    }

    #[test]
    fn html_escapes_user_controlled_text() {
        assert_eq!(
            escape_html("<b onclick='x'>"),
            "&lt;b onclick=&#39;x&#39;&gt;"
        );
    }
}
