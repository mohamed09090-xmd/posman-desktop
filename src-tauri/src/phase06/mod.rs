pub mod counts;
pub mod cump;
pub mod dto;
pub mod error;
pub mod fixed_point;
pub mod inventory;
pub mod projections;
pub mod purchasing;
pub mod reconciliation;
pub mod reservations;

#[cfg(test)]
mod tests;

use rusqlite::{params, OptionalExtension, Transaction, TransactionBehavior};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::phase05::{Phase05Service, Phase06AuthContext};

use self::{
    dto::{DocumentLineView, DocumentView, EntityResult},
    error::{Phase06Error, Phase06Result},
    fixed_point::line_totals,
};

const PHASE06_PERMISSIONS: &[(&str, &str, &str, &str, &str, bool)] = &[
    (
        "perm-stock-read",
        "stock.read",
        "inventory",
        "قراءة المخزون",
        "Consulter le stock",
        false,
    ),
    (
        "perm-stock-opening-post",
        "stock.opening.post",
        "inventory",
        "ترحيل المخزون الافتتاحي",
        "Valider le stock initial",
        true,
    ),
    (
        "perm-stock-adjust",
        "stock.adjust",
        "inventory",
        "تسوية المخزون",
        "Ajuster le stock",
        true,
    ),
    (
        "perm-stock-transfer",
        "stock.transfer",
        "inventory",
        "تحويل المخزون",
        "Transférer le stock",
        false,
    ),
    (
        "perm-stock-count",
        "stock.count",
        "inventory",
        "إدارة الجرد الفعلي",
        "Gérer les inventaires physiques",
        true,
    ),
    (
        "perm-stock-reservation-manage",
        "stock.reservation.manage",
        "inventory",
        "إدارة حجوزات المخزون",
        "Gérer les réservations de stock",
        false,
    ),
    (
        "perm-stock-reconcile",
        "stock.reconcile",
        "inventory",
        "مطابقة وإعادة بناء المخزون",
        "Rapprocher et reconstruire le stock",
        true,
    ),
    (
        "perm-stock-negative-override",
        "stock.negative.override",
        "inventory",
        "تجاوز المخزون السالب",
        "Autoriser le stock négatif",
        true,
    ),
    (
        "perm-purchase-order-confirm",
        "purchase_order.confirm",
        "purchases",
        "تأكيد أمر شراء",
        "Confirmer une commande fournisseur",
        false,
    ),
    (
        "perm-purchase-receipt-post",
        "purchase_receipt.post",
        "purchases",
        "ترحيل استلام شراء",
        "Valider une réception",
        true,
    ),
    (
        "perm-purchase-invoice-post",
        "purchase_invoice.post",
        "purchases",
        "ترحيل فاتورة مورد",
        "Valider une facture fournisseur",
        true,
    ),
    (
        "perm-purchase-return-post",
        "purchase_return.post",
        "purchases",
        "ترحيل مرتجع مشتريات",
        "Valider un retour fournisseur",
        true,
    ),
];

const ALL_PHASE06_CODES: &str = "'stock.read','stock.opening.post','stock.adjust','stock.transfer','stock.count','stock.reservation.manage','stock.reconcile','stock.negative.override','purchase_order.confirm','purchase_receipt.post','purchase_invoice.post','purchase_return.post'";
const STOCK_ROLE_CODES: &str = "'stock.read','stock.opening.post','stock.adjust','stock.transfer','stock.count','stock.reservation.manage','stock.reconcile'";
const PURCHASING_ROLE_CODES: &str = "'stock.read','purchase_order.confirm','purchase_receipt.post','purchase_invoice.post','purchase_return.post'";
const AUDITOR_ROLE_CODES: &str = "'stock.read'";

#[derive(Clone)]
pub struct Phase06Service {
    phase05: Phase05Service,
}

include!("service_impl.rs");
include!("idempotency_impl.rs");
include!("document_core.rs");
include!("document_lines.rs");
