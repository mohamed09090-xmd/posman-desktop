-- Safe deterministic POSMAN reference data. No company, user, password, tax rate, or account number is seeded.

INSERT OR IGNORE INTO roles (
    id, company_id, code, name_ar, name_fr, is_system, is_active,
    created_at, created_by, updated_at, updated_by, row_version
) VALUES
    ('role-owner', NULL, 'OWNER', 'المدير', 'Administrateur', 1, 1, '2026-07-28T00:00:00Z', NULL, '2026-07-28T00:00:00Z', NULL, 1),
    ('role-sales', NULL, 'SALES', 'مسؤول المبيعات', 'Responsable des ventes', 1, 1, '2026-07-28T00:00:00Z', NULL, '2026-07-28T00:00:00Z', NULL, 1),
    ('role-stock', NULL, 'STOCK', 'أمين المخزن', 'Magasinier', 1, 1, '2026-07-28T00:00:00Z', NULL, '2026-07-28T00:00:00Z', NULL, 1),
    ('role-purchasing', NULL, 'PURCHASING', 'مسؤول المشتريات', 'Responsable des achats', 1, 1, '2026-07-28T00:00:00Z', NULL, '2026-07-28T00:00:00Z', NULL, 1),
    ('role-accountant', NULL, 'ACCOUNTANT', 'المحاسب', 'Comptable', 1, 1, '2026-07-28T00:00:00Z', NULL, '2026-07-28T00:00:00Z', NULL, 1),
    ('role-auditor', NULL, 'AUDITOR', 'المراقب', 'Auditeur', 1, 1, '2026-07-28T00:00:00Z', NULL, '2026-07-28T00:00:00Z', NULL, 1);

INSERT OR IGNORE INTO permissions (
    id, code, domain, description_ar, description_fr, is_sensitive, created_at
) VALUES
    ('perm-product-create', 'product.create', 'catalog', 'إنشاء مادة', 'Créer un article', 0, '2026-07-28T00:00:00Z'),
    ('perm-product-update', 'product.update', 'catalog', 'تعديل مادة', 'Modifier un article', 0, '2026-07-28T00:00:00Z'),
    ('perm-pricing-override-below-cost', 'pricing.override_below_cost', 'catalog', 'تجاوز البيع تحت التكلفة', 'Autoriser la vente sous le coût', 1, '2026-07-28T00:00:00Z'),
    ('perm-partner-manage', 'partner.manage', 'partners', 'إدارة العملاء والموردين', 'Gérer les partenaires', 0, '2026-07-28T00:00:00Z'),
    ('perm-stock-read', 'stock.read', 'inventory', 'قراءة المخزون', 'Consulter le stock', 0, '2026-07-28T00:00:00Z'),
    ('perm-stock-opening-post', 'stock.opening.post', 'inventory', 'ترحيل المخزون الافتتاحي', 'Valider le stock initial', 1, '2026-07-28T00:00:00Z'),
    ('perm-stock-adjust', 'stock.adjust', 'inventory', 'تسوية المخزون', 'Ajuster le stock', 1, '2026-07-28T00:00:00Z'),
    ('perm-stock-transfer', 'stock.transfer', 'inventory', 'تحويل المخزون', 'Transférer le stock', 0, '2026-07-28T00:00:00Z'),
    ('perm-sales-order-confirm', 'sales_order.confirm', 'sales', 'تأكيد طلب عميل', 'Confirmer une commande client', 0, '2026-07-28T00:00:00Z'),
    ('perm-delivery-note-post', 'delivery_note.post', 'sales', 'ترحيل سند تسليم', 'Valider un bon de livraison', 1, '2026-07-28T00:00:00Z'),
    ('perm-sales-invoice-post', 'sales_invoice.post', 'sales', 'ترحيل فاتورة بيع', 'Valider une facture de vente', 1, '2026-07-28T00:00:00Z'),
    ('perm-purchase-order-confirm', 'purchase_order.confirm', 'purchases', 'تأكيد أمر شراء', 'Confirmer une commande fournisseur', 0, '2026-07-28T00:00:00Z'),
    ('perm-purchase-receipt-post', 'purchase_receipt.post', 'purchases', 'ترحيل استلام شراء', 'Valider une réception', 1, '2026-07-28T00:00:00Z'),
    ('perm-purchase-invoice-post', 'purchase_invoice.post', 'purchases', 'ترحيل فاتورة مورد', 'Valider une facture fournisseur', 1, '2026-07-28T00:00:00Z'),
    ('perm-payment-post', 'payment.post', 'payments', 'ترحيل قبض أو دفع', 'Valider un règlement', 1, '2026-07-28T00:00:00Z'),
    ('perm-journal-entry-post', 'journal_entry.post', 'accounting', 'ترحيل قيد محاسبي', 'Valider une écriture', 1, '2026-07-28T00:00:00Z'),
    ('perm-journal-entry-reverse', 'journal_entry.reverse', 'accounting', 'عكس قيد محاسبي', 'Contrepasser une écriture', 1, '2026-07-28T00:00:00Z'),
    ('perm-fiscal-period-close', 'fiscal_period.close', 'accounting', 'إغلاق فترة مالية', 'Clôturer une période', 1, '2026-07-28T00:00:00Z'),
    ('perm-audit-read', 'audit.read', 'audit', 'قراءة سجل العمليات', 'Consulter le journal d’audit', 1, '2026-07-28T00:00:00Z'),
    ('perm-backup-create', 'backup.create', 'backup', 'إنشاء نسخة احتياطية', 'Créer une sauvegarde', 1, '2026-07-28T00:00:00Z'),
    ('perm-backup-restore', 'backup.restore', 'backup', 'استعادة نسخة احتياطية', 'Restaurer une sauvegarde', 1, '2026-07-28T00:00:00Z'),
    ('perm-user-manage', 'user.manage', 'security', 'إدارة المستخدمين', 'Gérer les utilisateurs', 1, '2026-07-28T00:00:00Z'),
    ('perm-role-manage', 'role.manage', 'security', 'إدارة الأدوار والصلاحيات', 'Gérer les rôles et permissions', 1, '2026-07-28T00:00:00Z');

-- OWNER receives every current permission. The future application copies or resolves this system template during first-run setup.
INSERT OR IGNORE INTO role_permissions (id, company_id, role_id, permission_id, granted_at, granted_by)
SELECT 'rp-owner-' || permissions.id, NULL, 'role-owner', permissions.id, '2026-07-28T00:00:00Z', NULL
FROM permissions;

INSERT OR IGNORE INTO role_permissions (id, company_id, role_id, permission_id, granted_at, granted_by) VALUES
    ('rp-sales-product-create', NULL, 'role-sales', 'perm-product-create', '2026-07-28T00:00:00Z', NULL),
    ('rp-sales-product-update', NULL, 'role-sales', 'perm-product-update', '2026-07-28T00:00:00Z', NULL),
    ('rp-sales-partner-manage', NULL, 'role-sales', 'perm-partner-manage', '2026-07-28T00:00:00Z', NULL),
    ('rp-sales-stock-read', NULL, 'role-sales', 'perm-stock-read', '2026-07-28T00:00:00Z', NULL),
    ('rp-sales-order-confirm', NULL, 'role-sales', 'perm-sales-order-confirm', '2026-07-28T00:00:00Z', NULL),
    ('rp-sales-delivery-post', NULL, 'role-sales', 'perm-delivery-note-post', '2026-07-28T00:00:00Z', NULL),
    ('rp-sales-invoice-post', NULL, 'role-sales', 'perm-sales-invoice-post', '2026-07-28T00:00:00Z', NULL),
    ('rp-sales-payment-post', NULL, 'role-sales', 'perm-payment-post', '2026-07-28T00:00:00Z', NULL),

    ('rp-stock-stock-read', NULL, 'role-stock', 'perm-stock-read', '2026-07-28T00:00:00Z', NULL),
    ('rp-stock-opening-post', NULL, 'role-stock', 'perm-stock-opening-post', '2026-07-28T00:00:00Z', NULL),
    ('rp-stock-adjust', NULL, 'role-stock', 'perm-stock-adjust', '2026-07-28T00:00:00Z', NULL),
    ('rp-stock-transfer', NULL, 'role-stock', 'perm-stock-transfer', '2026-07-28T00:00:00Z', NULL),
    ('rp-stock-delivery-post', NULL, 'role-stock', 'perm-delivery-note-post', '2026-07-28T00:00:00Z', NULL),
    ('rp-stock-receipt-post', NULL, 'role-stock', 'perm-purchase-receipt-post', '2026-07-28T00:00:00Z', NULL),

    ('rp-purchasing-product-create', NULL, 'role-purchasing', 'perm-product-create', '2026-07-28T00:00:00Z', NULL),
    ('rp-purchasing-product-update', NULL, 'role-purchasing', 'perm-product-update', '2026-07-28T00:00:00Z', NULL),
    ('rp-purchasing-partner-manage', NULL, 'role-purchasing', 'perm-partner-manage', '2026-07-28T00:00:00Z', NULL),
    ('rp-purchasing-order-confirm', NULL, 'role-purchasing', 'perm-purchase-order-confirm', '2026-07-28T00:00:00Z', NULL),
    ('rp-purchasing-receipt-post', NULL, 'role-purchasing', 'perm-purchase-receipt-post', '2026-07-28T00:00:00Z', NULL),
    ('rp-purchasing-invoice-post', NULL, 'role-purchasing', 'perm-purchase-invoice-post', '2026-07-28T00:00:00Z', NULL),
    ('rp-purchasing-payment-post', NULL, 'role-purchasing', 'perm-payment-post', '2026-07-28T00:00:00Z', NULL),

    ('rp-accountant-journal-post', NULL, 'role-accountant', 'perm-journal-entry-post', '2026-07-28T00:00:00Z', NULL),
    ('rp-accountant-journal-reverse', NULL, 'role-accountant', 'perm-journal-entry-reverse', '2026-07-28T00:00:00Z', NULL),
    ('rp-accountant-period-close', NULL, 'role-accountant', 'perm-fiscal-period-close', '2026-07-28T00:00:00Z', NULL),
    ('rp-accountant-payment-post', NULL, 'role-accountant', 'perm-payment-post', '2026-07-28T00:00:00Z', NULL),
    ('rp-accountant-audit-read', NULL, 'role-accountant', 'perm-audit-read', '2026-07-28T00:00:00Z', NULL),
    ('rp-accountant-backup-create', NULL, 'role-accountant', 'perm-backup-create', '2026-07-28T00:00:00Z', NULL),

    ('rp-auditor-stock-read', NULL, 'role-auditor', 'perm-stock-read', '2026-07-28T00:00:00Z', NULL),
    ('rp-auditor-audit-read', NULL, 'role-auditor', 'perm-audit-read', '2026-07-28T00:00:00Z', NULL);
