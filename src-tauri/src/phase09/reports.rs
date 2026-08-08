use std::{collections::BTreeMap, fs, io::Write};

use rusqlite::{params_from_iter, types::Value as SqlValue, Row};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Wry};

use super::{
    checked_page,
    error::{Phase09Error, Phase09Result},
    models::{
        ExportResult, ReportColumn, ReportDescriptor, ReportPage, ReportRequest, ReportRow,
        ReportValue,
    },
    new_id, normalize_locale, now_iso, safe_component, Phase09Service,
};

const CSV_ROW_LIMIT: i64 = 100_000;
const PDF_ROW_LIMIT: i64 = 5_000;

impl Phase09Service {
    pub fn list_reports(&self, _: ()) -> Phase09Result<Vec<ReportDescriptor>> {
        self.authorize("reports.view")?;
        Ok(REPORT_IDS
            .iter()
            .map(|id| report_spec(id).descriptor)
            .collect())
    }

    pub fn run_report(&self, request: ReportRequest) -> Phase09Result<ReportPage> {
        self.authorize("reports.view")?;
        run_report_query(self, request, 200)
    }

    pub fn export_report_csv(&self, mut request: ReportRequest) -> Phase09Result<ExportResult> {
        let context = self.authorize("reports.export")?;
        request.page = 1;
        request.page_size = CSV_ROW_LIMIT;
        let page = run_report_query(self, request.clone(), CSV_ROW_LIMIT)?;
        if page.total_rows > CSV_ROW_LIMIT {
            return Err(Phase09Error::new(
                "EXPORT_ROW_LIMIT",
                "The report exceeds the 100000-row CSV limit. Narrow the filters and try again.",
                false,
            ));
        }
        let generated_at = now_iso()?;
        let relative_directory = "reports";
        let relative_name = format!(
            "{}-{}.csv",
            safe_component(&request.report_id.to_ascii_lowercase())?,
            safe_component(&new_id())?
        );
        let relative_path = format!("{relative_directory}/{relative_name}");
        let directory = self.paths.exports.join(relative_directory);
        fs::create_dir_all(&directory)?;
        let final_path = directory.join(&relative_name);
        let temporary_path = directory.join(format!(".{relative_name}.tmp"));
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary_path)?;
        file.write_all(&[0xEF, 0xBB, 0xBF])?;
        csv_row(&mut file, &["POSMAN REPORT EXPORT"])?;
        csv_row(&mut file, &["report", &request.report_id])?;
        csv_row(&mut file, &["generated_at", &generated_at])?;
        csv_row(&mut file, &["generated_by", &context.user_id])?;
        csv_row(&mut file, &["company_id", &context.company_id])?;
        csv_row(
            &mut file,
            &[
                "filters",
                &serde_json::to_string(&request).map_err(|_| Phase09Error::internal())?,
            ],
        )?;
        csv_row(&mut file, &[])?;
        let labels = page
            .columns
            .iter()
            .map(|column| {
                if request.locale == "ar-DZ" || request.locale == "ar" {
                    column.label_ar.as_str()
                } else {
                    column.label_fr.as_str()
                }
            })
            .collect::<Vec<_>>();
        csv_row(&mut file, &labels)?;
        for row in &page.rows {
            let cells = page
                .columns
                .iter()
                .map(|column| report_value_text(row.values.get(&column.key)))
                .collect::<Vec<_>>();
            let refs = cells.iter().map(String::as_str).collect::<Vec<_>>();
            csv_row(&mut file, &refs)?;
        }
        file.flush()?;
        file.sync_all()?;
        fs::rename(&temporary_path, &final_path).map_err(|error| {
            let _ = fs::remove_file(&temporary_path);
            Phase09Error::from(error)
        })?;
        export_result(&relative_path, &final_path)
    }

    pub fn export_report_pdf(
        &self,
        app: &AppHandle<Wry>,
        mut request: ReportRequest,
    ) -> Phase09Result<ExportResult> {
        self.authorize("reports.export")?;
        request.page = 1;
        request.page_size = PDF_ROW_LIMIT;
        let page = run_report_query(self, request.clone(), PDF_ROW_LIMIT)?;
        if page.total_rows > PDF_ROW_LIMIT {
            return Err(Phase09Error::new(
                "REPORT_PDF_LIMIT_EXCEEDED",
                "The report exceeds the 5000-row PDF limit. Narrow the filters or export CSV.",
                false,
            ));
        }
        let locale = normalize_locale(&request.locale)?;
        let spec = report_spec(&request.report_id);
        let title = if locale == "ar-DZ" {
            spec.descriptor.name_ar
        } else {
            spec.descriptor.name_fr
        };
        let (html, css) = render_report_html(&title, locale, &page);
        let relative_path = format!(
            "reports/{}-{}.pdf",
            safe_component(&request.report_id.to_ascii_lowercase())?,
            safe_component(&new_id())?
        );
        self.generate_managed_pdf_export(app, &html, &css, &relative_path)
    }
}

const REPORT_IDS: &[&str] = &[
    "SALES_SUMMARY",
    "SALES_BY_PRODUCT",
    "SALES_BY_CUSTOMER",
    "PURCHASES_SUMMARY",
    "PURCHASES_BY_SUPPLIER",
    "STOCK_ON_HAND",
    "STOCK_VALUATION",
    "STOCK_MOVEMENTS",
    "LOW_STOCK",
    "OPEN_RECEIVABLES",
    "OPEN_PAYABLES",
    "CASH_BANK_REGISTER",
    "TRIAL_BALANCE",
];

#[derive(Clone)]
struct ReportSpec {
    descriptor: ReportDescriptor,
    select_sql: String,
    suffix_sql: String,
    date_column: Option<&'static str>,
    warehouse_column: Option<&'static str>,
    partner_column: Option<&'static str>,
    product_column: Option<&'static str>,
    status_column: Option<&'static str>,
    columns: Vec<ReportColumn>,
    sort_fields: BTreeMap<String, String>,
    default_sort: String,
}

#[derive(Clone, Copy)]
struct ReportFilters {
    date: Option<&'static str>,
    warehouse: Option<&'static str>,
    partner: Option<&'static str>,
    product: Option<&'static str>,
    status: Option<&'static str>,
}

macro_rules! spec {
    ($id:expr,$ar:expr,$fr:expr,$select:expr,$suffix:expr,$date:expr,$warehouse:expr,$partner:expr,$product:expr,$status:expr,$columns:expr,$default_sort:expr $(,)?) => {
        build_spec(
            $id,
            ($ar, $fr),
            ($select, $suffix),
            ReportFilters {
                date: $date,
                warehouse: $warehouse,
                partner: $partner,
                product: $product,
                status: $status,
            },
            $columns,
            $default_sort,
        )
    };
}

fn run_report_query(
    service: &Phase09Service,
    request: ReportRequest,
    maximum_page_size: i64,
) -> Phase09Result<ReportPage> {
    normalize_locale(&request.locale)?;
    if !REPORT_IDS.contains(&request.report_id.as_str()) {
        return Err(Phase09Error::validation("Unsupported report identifier."));
    }
    let (page, page_size) = checked_page(request.page, request.page_size, maximum_page_size)?;
    validate_date_range(request.start_date.as_deref(), request.end_date.as_deref())?;
    let context = service.authorize("reports.view")?;
    let spec = report_spec(&request.report_id);
    let mut sql = spec.select_sql.clone();
    let mut values = vec![SqlValue::Text(context.company_id.clone())];
    append_filter(
        &mut sql,
        &mut values,
        spec.date_column,
        ">=",
        request.start_date.as_deref(),
    );
    append_filter(
        &mut sql,
        &mut values,
        spec.date_column,
        "<=",
        request.end_date.as_deref(),
    );
    append_filter(
        &mut sql,
        &mut values,
        spec.warehouse_column,
        "=",
        request.warehouse_id.as_deref(),
    );
    append_filter(
        &mut sql,
        &mut values,
        spec.partner_column,
        "=",
        request.partner_id.as_deref(),
    );
    append_filter(
        &mut sql,
        &mut values,
        spec.product_column,
        "=",
        request.product_id.as_deref(),
    );
    append_filter(
        &mut sql,
        &mut values,
        spec.status_column,
        "=",
        request.status.as_deref(),
    );
    sql.push_str(&spec.suffix_sql);
    let count_sql = format!("SELECT COUNT(*) FROM ({sql}) phase09_report_rows");
    let connection = service.phase05.phase09_open_maintenance()?;
    let total_rows = connection.query_row(&count_sql, params_from_iter(values.iter()), |row| {
        row.get(0)
    })?;
    let sort_field = request
        .sort_field
        .as_deref()
        .unwrap_or(spec.default_sort.as_str());
    let sort_sql = spec
        .sort_fields
        .get(sort_field)
        .ok_or_else(|| Phase09Error::validation("Unsupported report sort field."))?;
    let sort_direction = match request
        .sort_direction
        .as_deref()
        .unwrap_or("ASC")
        .to_ascii_uppercase()
        .as_str()
    {
        "ASC" => "ASC",
        "DESC" => "DESC",
        _ => return Err(Phase09Error::validation("Invalid report sort direction.")),
    };
    sql.push_str(&format!(
        " ORDER BY {sort_sql} {sort_direction} LIMIT ? OFFSET ?"
    ));
    values.push(SqlValue::Integer(page_size));
    values.push(SqlValue::Integer((page - 1) * page_size));
    let mut statement = connection.prepare(&sql)?;
    let rows = statement
        .query_map(params_from_iter(values.iter()), row_to_report)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ReportPage {
        report_id: request.report_id,
        columns: spec.columns,
        rows,
        page,
        page_size,
        total_rows,
        generated_at: now_iso()?,
    })
}

fn report_spec(id: &str) -> ReportSpec {
    match id {
        "SALES_SUMMARY" => spec!(
            id,
            "ملخص المبيعات",
            "Synthèse des ventes",
            "SELECT d.commercial_date AS businessDate,COUNT(*) AS documentCount,SUM(CASE WHEN d.document_type='SALES_CREDIT_NOTE' THEN -d.total_ht_minor ELSE d.total_ht_minor END) AS totalHtMinor,SUM(CASE WHEN d.document_type='SALES_CREDIT_NOTE' THEN -d.total_tax_minor ELSE d.total_tax_minor END) AS totalTaxMinor,SUM(CASE WHEN d.document_type='SALES_CREDIT_NOTE' THEN -d.total_ttc_minor ELSE d.total_ttc_minor END) AS totalTtcMinor FROM commercial_documents d WHERE d.company_id=? AND d.document_type IN ('SALES_INVOICE','SALES_CREDIT_NOTE') AND d.posting_status='POSTED'",
            " GROUP BY d.commercial_date",
            Some("d.commercial_date"), Some("d.warehouse_id"), Some("d.partner_id"), None, Some("d.workflow_status"),
            &[c("businessDate","التاريخ","Date","date"),c("documentCount","عدد المستندات","Documents","integer"),c("totalHtMinor","دون رسم","HT","moneyMinor"),c("totalTaxMinor","الرسم","TVA","moneyMinor"),c("totalTtcMinor","مع الرسم","TTC","moneyMinor")],
            "businessDate",
        ),
        "SALES_BY_PRODUCT" => spec!(
            id,"المبيعات حسب المنتج","Ventes par produit",
            "SELECT l.product_id AS productId,l.product_code_snapshot AS productCode,l.description_snapshot AS productName,SUM(CASE WHEN d.document_type='SALES_CREDIT_NOTE' THEN -l.quantity_scaled ELSE l.quantity_scaled END) AS quantityScaled,SUM(CASE WHEN d.document_type='SALES_CREDIT_NOTE' THEN -l.line_ht_minor ELSE l.line_ht_minor END) AS totalHtMinor,SUM(CASE WHEN d.document_type='SALES_CREDIT_NOTE' THEN -l.line_ttc_minor ELSE l.line_ttc_minor END) AS totalTtcMinor FROM commercial_documents d JOIN commercial_document_lines l ON l.document_id=d.id AND l.company_id=d.company_id WHERE d.company_id=? AND d.document_type IN ('SALES_INVOICE','SALES_CREDIT_NOTE') AND d.posting_status='POSTED'",
            " GROUP BY l.product_id,l.product_code_snapshot,l.description_snapshot",Some("d.commercial_date"),Some("l.warehouse_id"),Some("d.partner_id"),Some("l.product_id"),Some("d.workflow_status"),
            &[c("productCode","رمز المنتج","Code","text"),c("productName","المنتج","Produit","text"),c("quantityScaled","الكمية","Quantité","quantityScaled"),c("totalHtMinor","دون رسم","HT","moneyMinor"),c("totalTtcMinor","مع الرسم","TTC","moneyMinor")],"productCode"),
        "SALES_BY_CUSTOMER" => partner_spec(id,true),
        "PURCHASES_SUMMARY" => spec!(
            id,"ملخص المشتريات","Synthèse des achats",
            "SELECT d.commercial_date AS businessDate,COUNT(*) AS documentCount,SUM(CASE WHEN d.document_type='PURCHASE_RETURN' THEN -d.total_ht_minor ELSE d.total_ht_minor END) AS totalHtMinor,SUM(CASE WHEN d.document_type='PURCHASE_RETURN' THEN -d.total_tax_minor ELSE d.total_tax_minor END) AS totalTaxMinor,SUM(CASE WHEN d.document_type='PURCHASE_RETURN' THEN -d.total_ttc_minor ELSE d.total_ttc_minor END) AS totalTtcMinor FROM commercial_documents d WHERE d.company_id=? AND d.document_type IN ('PURCHASE_INVOICE','PURCHASE_RETURN') AND d.posting_status='POSTED'",
            " GROUP BY d.commercial_date",Some("d.commercial_date"),Some("d.warehouse_id"),Some("d.partner_id"),None,Some("d.workflow_status"),
            &[c("businessDate","التاريخ","Date","date"),c("documentCount","عدد المستندات","Documents","integer"),c("totalHtMinor","دون رسم","HT","moneyMinor"),c("totalTaxMinor","الرسم","TVA","moneyMinor"),c("totalTtcMinor","مع الرسم","TTC","moneyMinor")],"businessDate"),
        "PURCHASES_BY_SUPPLIER" => partner_spec(id,false),
        "STOCK_ON_HAND" => stock_spec(id,false,false),
        "STOCK_VALUATION" => stock_spec(id,true,false),
        "LOW_STOCK" => stock_spec(id,false,true),
        "STOCK_MOVEMENTS" => spec!(
            id,"حركات المخزون","Mouvements de stock",
            "SELECT m.occurred_at AS occurredAt,m.business_date AS businessDate,p.code AS productCode,p.name_ar AS productAr,p.name_fr AS productFr,w.code AS warehouseCode,m.movement_type AS movementType,m.quantity_delta_scaled AS quantityDeltaScaled,m.quantity_after_scaled AS quantityAfterScaled,m.extended_cost_minor AS extendedCostMinor FROM stock_movements m JOIN products p ON p.id=m.product_id AND p.company_id=m.company_id JOIN warehouses w ON w.id=m.warehouse_id AND w.company_id=m.company_id WHERE m.company_id=?",
            "",Some("m.business_date"),Some("m.warehouse_id"),None,Some("m.product_id"),Some("m.movement_type"),
            &[c("businessDate","التاريخ","Date","date"),c("productCode","رمز المنتج","Code","text"),c("productAr","المنتج","Produit AR","text"),c("productFr","المنتج بالفرنسية","Produit","text"),c("warehouseCode","المخزن","Dépôt","text"),c("movementType","الحركة","Mouvement","text"),c("quantityDeltaScaled","التغير","Variation","quantityScaled"),c("quantityAfterScaled","الرصيد","Solde","quantityScaled"),c("extendedCostMinor","القيمة","Valeur","moneyMinor")],"occurredAt"),
        "OPEN_RECEIVABLES" => open_balance_spec(id,true),
        "OPEN_PAYABLES" => open_balance_spec(id,false),
        "CASH_BANK_REGISTER" => spec!(
            id,"سجل الصندوق والبنك","Registre caisse et banque",
            "SELECT pay.commercial_date AS businessDate,pay.payment_number AS paymentNumber,pay.payment_kind AS paymentKind,pm.name_ar AS methodAr,pm.name_fr AS methodFr,pay.external_reference AS externalReference,CASE WHEN pay.payment_kind='RECEIPT' THEN pay.amount_minor ELSE -pay.amount_minor END AS signedAmountMinor,pay.status AS status FROM payments pay JOIN payment_methods pm ON pm.id=pay.payment_method_id AND pm.company_id=pay.company_id WHERE pay.company_id=? AND pay.status NOT IN ('DRAFT','CANCELLED')",
            "",Some("pay.commercial_date"),None,Some("pay.partner_id"),None,Some("pay.status"),
            &[c("businessDate","التاريخ","Date","date"),c("paymentNumber","رقم الدفع","Paiement","text"),c("paymentKind","النوع","Type","text"),c("methodAr","الطريقة","Mode AR","text"),c("methodFr","الطريقة بالفرنسية","Mode","text"),c("externalReference","المرجع","Référence","text"),c("signedAmountMinor","المبلغ","Montant","moneyMinor"),c("status","الحالة","Statut","text")],"businessDate"),
        "TRIAL_BALANCE" => spec!(
            id,"ميزان المراجعة","Balance générale",
            "SELECT a.id AS accountId,a.code AS accountCode,a.name_ar AS accountAr,a.name_fr AS accountFr,a.account_type AS accountType,COALESCE(SUM(CASE WHEN e.status='POSTED' THEN l.debit_minor ELSE 0 END),0) AS debitMinor,COALESCE(SUM(CASE WHEN e.status='POSTED' THEN l.credit_minor ELSE 0 END),0) AS creditMinor,COALESCE(SUM(CASE WHEN e.status='POSTED' THEN l.debit_minor-l.credit_minor ELSE 0 END),0) AS balanceMinor FROM accounts a LEFT JOIN journal_entry_lines l ON l.account_id=a.id AND l.company_id=a.company_id LEFT JOIN journal_entries e ON e.id=l.journal_entry_id AND e.company_id=a.company_id WHERE a.company_id=?",
            " GROUP BY a.id,a.code,a.name_ar,a.name_fr,a.account_type",Some("e.entry_date"),None,Some("l.partner_id"),Some("l.product_id"),Some("a.account_type"),
            &[c("accountCode","رمز الحساب","Compte","text"),c("accountAr","اسم الحساب","Libellé AR","text"),c("accountFr","الاسم بالفرنسية","Libellé","text"),c("accountType","النوع","Type","text"),c("debitMinor","مدين","Débit","moneyMinor"),c("creditMinor","دائن","Crédit","moneyMinor"),c("balanceMinor","الرصيد","Solde","moneyMinor")],"accountCode"),
        _ => invalid_spec(id),
    }
}

fn build_spec(
    id: &str,
    names: (&str, &str),
    sql: (&str, &str),
    filters: ReportFilters,
    columns: &[ReportColumn],
    default_sort: &str,
) -> ReportSpec {
    let mut sort_fields = BTreeMap::new();
    for column in columns {
        sort_fields.insert(column.key.clone(), column.key.clone());
    }
    ReportSpec {
        descriptor: ReportDescriptor {
            report_id: id.to_owned(),
            name_ar: names.0.to_owned(),
            name_fr: names.1.to_owned(),
            supports_date_range: filters.date.is_some(),
            supports_warehouse: filters.warehouse.is_some(),
            supports_partner: filters.partner.is_some(),
            supports_product: filters.product.is_some(),
            supports_status: filters.status.is_some(),
        },
        select_sql: sql.0.to_owned(),
        suffix_sql: sql.1.to_owned(),
        date_column: filters.date,
        warehouse_column: filters.warehouse,
        partner_column: filters.partner,
        product_column: filters.product,
        status_column: filters.status,
        columns: columns.to_vec(),
        sort_fields,
        default_sort: default_sort.to_owned(),
    }
}

fn partner_spec(id: &str, sales: bool) -> ReportSpec {
    let (ar, fr, types) = if sales {
        (
            "المبيعات حسب الزبون",
            "Ventes par client",
            "('SALES_INVOICE','SALES_CREDIT_NOTE')",
        )
    } else {
        (
            "المشتريات حسب المورد",
            "Achats par fournisseur",
            "('PURCHASE_INVOICE','PURCHASE_RETURN')",
        )
    };
    let sql = format!("SELECT d.partner_id AS partnerId,COALESCE(p.display_name_ar,p.legal_name,'—') AS partnerAr,COALESCE(p.display_name_fr,p.legal_name,'—') AS partnerFr,COUNT(*) AS documentCount,SUM(CASE WHEN d.document_type IN ('SALES_CREDIT_NOTE','PURCHASE_RETURN') THEN -d.total_ht_minor ELSE d.total_ht_minor END) AS totalHtMinor,SUM(CASE WHEN d.document_type IN ('SALES_CREDIT_NOTE','PURCHASE_RETURN') THEN -d.total_ttc_minor ELSE d.total_ttc_minor END) AS totalTtcMinor FROM commercial_documents d LEFT JOIN partners p ON p.id=d.partner_id AND p.company_id=d.company_id WHERE d.company_id=? AND d.document_type IN {types} AND d.posting_status='POSTED'");
    spec!(
        id,
        ar,
        fr,
        sql.as_str(),
        " GROUP BY d.partner_id,p.display_name_ar,p.display_name_fr,p.legal_name",
        Some("d.commercial_date"),
        Some("d.warehouse_id"),
        Some("d.partner_id"),
        None,
        Some("d.workflow_status"),
        &[
            c("partnerAr", "الشريك", "Partenaire AR", "text"),
            c("partnerFr", "الشريك بالفرنسية", "Partenaire", "text"),
            c("documentCount", "عدد المستندات", "Documents", "integer"),
            c("totalHtMinor", "دون رسم", "HT", "moneyMinor"),
            c("totalTtcMinor", "مع الرسم", "TTC", "moneyMinor"),
        ],
        "partnerAr",
    )
}

fn stock_spec(id: &str, valuation: bool, low_only: bool) -> ReportSpec {
    let (ar, fr) = match (valuation, low_only) {
        (true, _) => ("تقييم المخزون", "Valorisation du stock"),
        (_, true) => ("المخزون المنخفض", "Stock faible"),
        _ => ("المخزون المتاح", "Stock disponible"),
    };
    let valuation_columns = if valuation {
        ",b.average_cost_scaled AS averageCostScaled,CAST((b.on_hand_scaled*b.average_cost_scaled)/100000000 AS INTEGER) AS valueMinor"
    } else {
        ""
    };
    let low = if low_only {
        " AND b.available_scaled<p.minimum_stock_scaled"
    } else {
        ""
    };
    let sql = format!("SELECT b.product_id AS productId,p.code AS productCode,p.name_ar AS productAr,p.name_fr AS productFr,w.code AS warehouseCode,b.on_hand_scaled AS onHandScaled,b.reserved_scaled AS reservedScaled,b.available_scaled AS availableScaled,p.minimum_stock_scaled AS minimumStockScaled{valuation_columns} FROM stock_balances b JOIN products p ON p.id=b.product_id AND p.company_id=b.company_id JOIN warehouses w ON w.id=b.warehouse_id AND w.company_id=b.company_id WHERE b.company_id=?{low}");
    let mut columns = vec![
        c("productCode", "رمز المنتج", "Code", "text"),
        c("productAr", "المنتج", "Produit AR", "text"),
        c("productFr", "المنتج بالفرنسية", "Produit", "text"),
        c("warehouseCode", "المخزن", "Dépôt", "text"),
        c("onHandScaled", "المخزون", "Stock", "quantityScaled"),
        c("reservedScaled", "محجوز", "Réservé", "quantityScaled"),
        c("availableScaled", "متاح", "Disponible", "quantityScaled"),
        c(
            "minimumStockScaled",
            "الحد الأدنى",
            "Minimum",
            "quantityScaled",
        ),
    ];
    if valuation {
        columns.push(c(
            "averageCostScaled",
            "متوسط التكلفة",
            "Coût moyen",
            "moneyScaled",
        ));
        columns.push(c("valueMinor", "القيمة", "Valeur", "moneyMinor"));
    }
    spec!(
        id,
        ar,
        fr,
        sql.as_str(),
        "",
        None,
        Some("b.warehouse_id"),
        None,
        Some("b.product_id"),
        None,
        &columns,
        "productCode",
    )
}

fn open_balance_spec(id: &str, receivable: bool) -> ReportSpec {
    let (ar, fr, document_type) = if receivable {
        ("الذمم المدينة", "Créances ouvertes", "SALES_INVOICE")
    } else {
        ("الذمم الدائنة", "Dettes ouvertes", "PURCHASE_INVOICE")
    };
    let sql = format!("SELECT d.id AS documentId,d.document_number AS documentNumber,d.commercial_date AS documentDate,d.due_date AS dueDate,d.partner_id AS partnerId,COALESCE(p.display_name_ar,p.legal_name,'—') AS partnerAr,COALESCE(p.display_name_fr,p.legal_name,'—') AS partnerFr,d.total_ttc_minor AS originalMinor,COALESCE(SUM(CASE WHEN pa.allocation_status='ACTIVE' THEN pa.allocated_amount_minor ELSE 0 END),0) AS allocatedMinor,d.total_ttc_minor-COALESCE(SUM(CASE WHEN pa.allocation_status='ACTIVE' THEN pa.allocated_amount_minor ELSE 0 END),0) AS openMinor FROM commercial_documents d LEFT JOIN partners p ON p.id=d.partner_id AND p.company_id=d.company_id LEFT JOIN payment_allocations pa ON pa.document_id=d.id AND pa.company_id=d.company_id WHERE d.company_id=? AND d.document_type='{document_type}' AND d.posting_status='POSTED'");
    spec!(id,ar,fr,sql.as_str()," GROUP BY d.id,d.document_number,d.commercial_date,d.due_date,d.partner_id,p.display_name_ar,p.display_name_fr,p.legal_name HAVING openMinor>0",Some("d.commercial_date"),Some("d.warehouse_id"),Some("d.partner_id"),None,Some("d.workflow_status"),&[c("documentNumber","رقم المستند","Document","text"),c("documentDate","التاريخ","Date","date"),c("dueDate","الاستحقاق","Échéance","date"),c("partnerAr","الشريك","Partenaire AR","text"),c("partnerFr","الشريك بالفرنسية","Partenaire","text"),c("originalMinor","الأصل","Montant","moneyMinor"),c("allocatedMinor","مسدد","Réglé","moneyMinor"),c("openMinor","المتبقي","Ouvert","moneyMinor")],"dueDate")
}

fn invalid_spec(id: &str) -> ReportSpec {
    spec!(
        id,
        "غير مدعوم",
        "Non pris en charge",
        "SELECT 1 AS invalid WHERE 0",
        "",
        None,
        None,
        None,
        None,
        None,
        &[c("invalid", "غير مدعوم", "Non pris en charge", "text")],
        "invalid",
    )
}

fn c(key: &str, ar: &str, fr: &str, kind: &str) -> ReportColumn {
    ReportColumn {
        key: key.to_owned(),
        label_ar: ar.to_owned(),
        label_fr: fr.to_owned(),
        kind: kind.to_owned(),
    }
}

fn append_filter(
    sql: &mut String,
    values: &mut Vec<SqlValue>,
    column: Option<&str>,
    operator: &str,
    value: Option<&str>,
) {
    if let (Some(column), Some(value)) = (
        column,
        value.map(str::trim).filter(|value| !value.is_empty()),
    ) {
        sql.push_str(&format!(" AND {column}{operator}?"));
        values.push(SqlValue::Text(value.to_owned()));
    }
}

fn validate_date_range(start: Option<&str>, end: Option<&str>) -> Phase09Result<()> {
    if let (Some(start), Some(end)) = (start, end) {
        if start > end {
            return Err(Phase09Error::validation(
                "Report start date must not be after end date.",
            ));
        }
    }
    Ok(())
}

fn row_to_report(row: &Row<'_>) -> rusqlite::Result<ReportRow> {
    let reference = row.as_ref();
    let mut values = BTreeMap::new();
    for index in 0..reference.column_count() {
        let key = reference.column_name(index)?.to_owned();
        let value = match row.get_ref(index)? {
            rusqlite::types::ValueRef::Null => ReportValue::Null,
            rusqlite::types::ValueRef::Integer(value) => ReportValue::Integer(value),
            rusqlite::types::ValueRef::Text(value) => {
                ReportValue::Text(String::from_utf8_lossy(value).into_owned())
            }
            rusqlite::types::ValueRef::Real(_) | rusqlite::types::ValueRef::Blob(_) => {
                return Err(rusqlite::Error::InvalidColumnType(
                    index,
                    key,
                    rusqlite::types::Type::Real,
                ));
            }
        };
        values.insert(key, value);
    }
    Ok(ReportRow { values })
}

fn report_value_text(value: Option<&ReportValue>) -> String {
    match value {
        Some(ReportValue::Text(value)) => value.clone(),
        Some(ReportValue::Integer(value)) => value.to_string(),
        Some(ReportValue::Boolean(value)) => value.to_string(),
        Some(ReportValue::Null) | None => String::new(),
    }
}

fn csv_row(file: &mut fs::File, cells: &[&str]) -> Phase09Result<()> {
    for (index, cell) in cells.iter().enumerate() {
        if index > 0 {
            file.write_all(b";")?;
        }
        file.write_all(neutralize_csv(cell).as_bytes())?;
    }
    file.write_all(b"\r\n")?;
    Ok(())
}

pub fn neutralize_csv(value: &str) -> String {
    csv_cell(value)
}

fn csv_cell(value: &str) -> String {
    let cleaned = value
        .chars()
        .filter(|character| !character.is_control() || matches!(character, '\t' | '\n' | '\r'))
        .collect::<String>()
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let neutralized = if matches!(
        cleaned.trim_start().chars().next(),
        Some('=' | '+' | '-' | '@')
    ) {
        format!("'{cleaned}")
    } else {
        cleaned
    };
    format!("\"{}\"", neutralized.replace('"', "\"\""))
}

fn export_result(relative_path: &str, path: &std::path::Path) -> Phase09Result<ExportResult> {
    let bytes = fs::read(path)?;
    Ok(ExportResult {
        relative_path: relative_path.to_owned(),
        sha256: format!("{:x}", Sha256::digest(&bytes)),
        size_bytes: i64::try_from(bytes.len()).map_err(|_| Phase09Error::internal())?,
    })
}

fn render_report_html(title: &str, locale: &str, page: &ReportPage) -> (String, String) {
    let direction = if locale == "ar-DZ" { "rtl" } else { "ltr" };
    let mut headings = String::new();
    for column in &page.columns {
        let label = if locale == "ar-DZ" {
            &column.label_ar
        } else {
            &column.label_fr
        };
        headings.push_str(&format!(
            "<th>{}</th>",
            super::rendering::escape_html(label)
        ));
    }
    let mut rows = String::new();
    for row in &page.rows {
        rows.push_str("<tr>");
        for column in &page.columns {
            rows.push_str(&format!(
                "<td>{}</td>",
                super::rendering::escape_html(&report_value_text(row.values.get(&column.key)))
            ));
        }
        rows.push_str("</tr>");
    }
    let html=format!("<!doctype html><html lang=\"{locale}\" dir=\"{direction}\"><head><meta charset=\"utf-8\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; script-src 'none'; connect-src 'none'; object-src 'none'; frame-src 'none'\"><title>{}</title></head><body><h1>{}</h1><p>{}</p><table><thead><tr>{headings}</tr></thead><tbody>{rows}</tbody></table></body></html>",super::rendering::escape_html(title),super::rendering::escape_html(title),super::rendering::escape_html(&page.generated_at));
    let css="@page{size:A4 landscape;margin:10mm}*{box-sizing:border-box}body{font-family:'Noto Sans Arabic','Segoe UI',sans-serif;font-size:10px;color:#111827}h1{font-size:20px}table{width:100%;border-collapse:collapse;table-layout:fixed}thead{display:table-header-group}tr{break-inside:avoid}th,td{border:1px solid #d1d5db;padding:5px;overflow-wrap:anywhere}th{background:#f3f4f6}".to_owned();
    (html, css)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_required_reports_have_typed_specs() {
        for id in REPORT_IDS {
            assert_eq!(report_spec(id).descriptor.report_id, *id);
        }
    }
    #[test]
    fn spreadsheet_formula_prefixes_are_neutralized() {
        assert_eq!(csv_cell(" =1+1"), "\"' =1+1\"");
    }

    #[test]
    fn dynamic_report_specs_remain_owned_and_complete() {
        for _ in 0..100 {
            for id in REPORT_IDS {
                let specification = report_spec(id);
                assert!(!specification.select_sql.is_empty());
                assert!(specification
                    .sort_fields
                    .contains_key(specification.default_sort.as_str()));
            }
        }
    }
}
