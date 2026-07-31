# POSMAN — PHASE 02 Runtime Foundation Execution Pack

## 0. هوية المهمة

أنت وكيل التنفيذ المسؤول حصريًا عن **PHASE 02 — Local Runtime Foundation** في مشروع POSMAN.

هذه مهمة تنفيذ فعلية داخل GitHub وليست مهمة تخطيط أو شرح. نفّذ الكود، الاختبارات، التوثيق، وCI ضمن الحدود التالية، ثم افتح Draft Pull Request واتركه دون دمج للمراجعة الخارجية.

### المستودع

```text
https://github.com/mohamed09090-xmd/posman-desktop
```

### baseline المعتمد حصريًا

```text
a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9
```

هذا الـSHA يحتوي:

- PHASE 01 SQLite data foundation المعتمدة.
- Bootstrap Gate المعتمد لـTauri 2 + React + TypeScript + Vite.
- عقد الملكية للعمل المتوازي.

### الفرع المطلوب

```text
phase/02-runtime-foundation
```

### Draft PR المطلوب

```text
[Phase 02] POSMAN local runtime foundation
```

يستهدف:

```text
main
```

لا تحوّله إلى Ready، ولا تدمجه، ولا تستخدم auto-merge.

---

## 1. Bootstrap Gate قبل أي تعديل

نفّذ فحوصًا read-only قبل إنشاء الفرع أو تعديل الملفات:

1. تأكد أن `main` يساوي حرفيًا:

   ```text
   a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9
   ```

2. اقرأ كاملًا:

   ```text
   AGENTS.md
   docs/spec/POSMAN-Blueprint-v1.md
   docs/architecture/parallel-wave-02-contract.md
   docs/architecture/database-decisions.md
   docs/architecture/migration-policy.md
   docs/architecture/desktop-shell.md
   docs/BOOTSTRAP-GATE-02-REPORT.md
   docs/PHASE-01-REPORT.md
   ```

3. افحص:

   ```text
   database/migrations/**
   database/seed/reference_data.sql
   database/tests/invariants.sql
   scripts/verify_schema.py
   src-tauri/Cargo.toml
   src-tauri/Cargo.lock
   src-tauri/build.rs
   src-tauri/src/lib.rs
   src-tauri/tauri.conf.json
   ```

4. شغّل baseline checks المتاحة قبل التعديل:

   ```text
   python scripts/verify_schema.py
   npm ci
   npm run typecheck
   npm run build
   cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
   cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
   cargo test --manifest-path src-tauri/Cargo.toml --all-targets --locked
   ```

إذا لم يطابق `main` الـSHA المعتمد، أو كان الفرع المطلوب موجودًا برأس غير متوقع، أو فشل baseline بسبب عيب موجود قبل عملك، **توقف وقدّم تقرير blocker**. لا تصلح عيبًا خارج PHASE 02 دون اعتماد معماري.

---

## 2. الهدف الدقيق للمرحلة

تحويل SQLite foundation من مخطط ساكن إلى **runtime محلي آمن** يستطيع تطبيق Tauri تشغيله تلقائيًا دون تثبيت قاعدة بيانات أو خادم خارجي.

عند تشغيل POSMAN لأول مرة يجب أن يستطيع Rust:

1. حل مجلد البيانات المحلي بطريقة صحيحة لكل نظام.
2. إنشاء بنية مجلدات POSMAN المحلية.
3. إنشاء ملف SQLite عند عدم وجوده.
4. فتح كل اتصال بعقد PRAGMA المعتمد.
5. تطبيق migrations الأربع الحالية بالترتيب داخل معاملات ذرية.
6. تسجيل الاسم والنسخة وSHA-256 في `app_migrations`.
7. رفض migration معدلة، سجل ناقص، أو schema أحدث غير معروف.
8. تطبيق seed المرجعي الآمن بصورة idempotent.
9. فحص سلامة المفاتيح الأجنبية بعد التهيئة.
10. توفير Tauri command للقراءة فقط يعيد حالة runtime دون كشف مسار محلي حساس أو تنفيذ SQL من الواجهة.

هذه المرحلة هي **runtime foundation فقط**؛ ليست مرحلة إعداد المؤسسة أو المستخدمين أو العمليات التجارية.

---

## 3. حدود الملكية الإلزامية

### الملفات التي تملكها PHASE 02

يجوز إنشاء أو تعديل:

```text
src-tauri/**
src/platform/tauri/**
.github/workflows/runtime-ci.yml
docs/architecture/runtime-*.md
docs/PHASE-02-REPORT.md
```

يجوز تعديل:

```text
src-tauri/Cargo.toml
src-tauri/Cargo.lock
```

لإضافة أقل مجموعة dependencies لازمة فقط.

### ممنوع تعديلها

```text
database/**
scripts/verify_schema.py
README.md
docs/spec/POSMAN-Blueprint-v1.md
docs/PHASE-01-REPORT.md
docs/BOOTSTRAP-GATE-02-REPORT.md
docs/architecture/accounting-posting.md
docs/architecture/data-dictionary.md
docs/architecture/database-decisions.md
docs/architecture/erd.md
docs/architecture/migration-policy.md
docs/architecture/parallel-wave-02-contract.md
docs/architecture/desktop-shell.md
src/app/**
src/components/**
src/i18n/**
src/styles/**
src/features/**
package.json
package-lock.json
vite.config.ts
src/bootstrap/bootstrap.css
.github/workflows/schema-ci.yml
.github/workflows/desktop-bootstrap-ci.yml
```

### الملفات المشتركة المجمّدة

ممنوع تعديل:

```text
src/main.tsx
index.html
tsconfig.json
tsconfig.app.json
tsconfig.node.json
AGENTS.md
.gitignore
```

لا يوجد استثناء ضمني. إذا احتجت فعلًا إلى ملف خارج الملكية، توقف قبل تعديله واطلب **Integration Patch Decision**.

---

## 4. القرارات التقنية الإلزامية

### 4.1 SQLite

- استخدم `rusqlite` مباشرة من Rust مع ميزة SQLite المدمج `bundled`.
- لا تستخدم Tauri SQL plugin.
- لا تستخدم SQLite executable خارجيًا.
- لا تستخدم sidecar أو server أو Docker.
- لا يحتاج المستخدم إلى تحميل قاعدة بيانات أو runtime منفصل.
- قاعدة البيانات ملف محلي واحد باسم واضح مثل:

  ```text
  posman.sqlite3
  ```

### 4.2 مسار البيانات

حل المجلد عبر Tauri/OS path APIs، ولا تكتب اسم مستخدم أو مسارًا مطلقًا داخل الكود.

المسار المنطقي المطلوب على Windows:

```text
%LOCALAPPDATA%\POSMAN\
├── data\
├── backups\
├── documents\
├── templates\
└── logs\
```

ملف قاعدة البيانات:

```text
%LOCALAPPDATA%\POSMAN\data\posman.sqlite3
```

أنشئ طبقة pure/testable تستقبل root صريحًا في الاختبارات، وadapter منفصلًا يحل root الحقيقي عبر Tauri في التطبيق. لا تجعل الاختبارات تكتب في مجلد المستخدم الحقيقي.

### 4.3 عقد الاتصال

كل اتصال SQLite ينفذ ويتحقق من:

```sql
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
```

للاتصال التشغيلي writable:

```sql
PRAGMA journal_mode = WAL;
```

يجب:

- التحقق أن `foreign_keys` أصبح `1`.
- تسجيل/إرجاع journal mode الفعلي بدل افتراض أن WAL نجح.
- التعامل بوضوح مع حالات الاختبار in-memory أو filesystem لا يدعم WAL.
- عدم تغيير PRAGMA داخل transaction بصورة غير فعالة.

### 4.4 migration catalog

المصدر الوحيد هو الملفات الحالية:

```text
database/migrations/0001_system_company_security.sql
database/migrations/0002_reference_catalog_partners.sql
database/migrations/0003_commerce_inventory.sql
database/migrations/0004_accounting_documents_audit.sql
```

ادمج bytes داخل binary وقت البناء باستخدام `include_str!`/`include_bytes!` بمسار مشتق من `CARGO_MANIFEST_DIR`. لا تنسخ SQL إلى `src-tauri` ولا تنشئ migration خامسة.

لكل migration خزّن في catalog:

- النسخة: `0001` … `0004`.
- الاسم المنطقي من filename.
- محتوى SQL.
- SHA-256 محسوب من bytes نفسها.

يجب أن يتطابق حساب SHA-256 مع:

```python
hashlib.sha256(sql.encode("utf-8")).hexdigest()
```

المستخدم في `scripts/verify_schema.py`.

### 4.5 خوارزمية migrations

طبّق ما يلي:

1. الملفات مرتبة، متصلة، ولا يوجد تكرار.
2. اقرأ ledger الموجود إن كان `app_migrations` موجودًا.
3. السجلات المطبقة يجب أن تكون prefix متصلًا من catalog.
4. كل سجل موجود يجب أن يطابق:
   - `id`
   - `version`
   - `name`
   - `checksum_sha256`
5. migration مجهولة أحدث من supported catalog تؤدي إلى رفض business startup.
6. checksum mismatch أو gap أو partial ledger خطأ fatal واضح، وليس warning.
7. كل migration غير مطبقة تنفذ داخل `BEGIN IMMEDIATE`/transaction واحدة.
8. SQL وتسجيل صف `app_migrations` يلتزمان أو يتراجعان معًا.
9. migration الأولى تنشئ `app_migrations` وتُسجّل داخل نفس transaction.
10. عند أول فشل:
    - rollback.
    - لا تطبق migrations اللاحقة.
    - لا تواصل تشغيل الأعمال على schema جزئية.
11. لا تنفذ down migrations.
12. لا تعدل migration مقبولة.

### 4.6 seed المرجعي

ادمج ونفّذ:

```text
database/seed/reference_data.sql
```

بعد اكتمال migrations داخل transaction مستقلة.

متطلبات:

- idempotent.
- لا ينشئ شركة أو مستخدمًا أو كلمة مرور أو ضريبة أو حسابًا محاسبيًا.
- التشغيل مرتين لا يكرر الأدوار والصلاحيات.
- فشل seed يتراجع بالكامل ولا يترك حالة جزئية.

### 4.7 فحص ما بعد التهيئة

بعد migrations والـseed:

```sql
PRAGMA foreign_key_check;
```

يجب أن يعيد صفر صفوف. أي نتيجة تمنع إعلان runtime جاهزًا.

تحقق كذلك من:

- أربع migrations مسجلة.
- schema version الحالية `0004`.
- الجداول المتوقعة موجودة.
- لا تنشئ schema ثانية موازية.

### 4.8 الأخطاء

أنشئ error model typed يفصل على الأقل:

- path resolution/creation failure.
- database open/configuration failure.
- unsupported/newer schema.
- migration checksum mismatch.
- non-contiguous/partial ledger.
- migration execution failure.
- seed failure.
- integrity failure.

الأخطاء الداخلية تحتفظ بالسياق للمطور، لكن Tauri command لا يعيد SQL خامًا أو مسار قاعدة البيانات الكامل أو أسرارًا إلى الواجهة.

لا تستخدم panic للأخطاء التشغيلية المتوقعة. `expect` مقبول فقط عند نقطة تشغيل التطبيق بعد تحويل الخطأ إلى فشل startup واضح.

### 4.9 العمليات المتزامنة

- لا تخزن `rusqlite::Connection` واحدة عالميًا إن كان ذلك يخرق `Send/Sync`.
- يمكن أن تحتفظ state بمسار وخدمة تفتح اتصالات مضبوطة عند الحاجة.
- نفّذ عمليات SQLite blocking خارج UI thread عند استدعائها من Tauri command، مثل `spawn_blocking`.
- initialization في startup يجب أن يكون حتميًا ولا يسمح بأمر business قبل readiness.

### 4.10 Tauri integration

حدّث `configure_application` بصورة تحافظ على generic runtime والاختبار الحالي.

أضف:

- managed runtime state.
- setup initialization.
- invoke handler لأمر القراءة فقط.

اسم الأمر المقترح:

```text
get_runtime_status
```

العقد serializable بـ`camelCase` ويعيد فقط:

```text
databaseReady
schemaVersion
migrationCount
foreignKeysEnabled
journalMode
```

لا يعيد:

- database absolute path.
- SQL.
- أسماء مستخدمين.
- بيانات شركة.

حافظ على الاختبار الحقيقي:

```text
application_setup_builds_with_mock_runtime
```

ولا تستبدله باختبار نصي، ولا تتجاهله على Windows. إذا احتاج setup injection للاختبار، صمّم constructor/test adapter واضحًا دون environment global متسابق.

### 4.11 Dependencies

أضف أقل dependencies لازمة فقط، مع تفضيل مكتبات Rust maintained ومتوافقة مع:

```text
rust-version = 1.85
```

متوقع عادة:

- `rusqlite` مع `bundled`.
- `sha2`.
- `serde` derive.
- error crate مثل `thiserror`.
- مكتبة وقت UTC maintained عند الحاجة.
- `tempfile` كـdev-dependency.

لا تضف ORM، async database framework، Tokio كاملًا، logging stack كبيرًا، UUID، Decimal، password hashing أو dependencies لأعمال لم تبدأ.

احتفظ بـ`Cargo.lock` محدثًا و`--locked` ناجحًا.

---

## 5. بنية مقترحة وليست إلزامية حرفيًا

استخدم حدودًا واضحة شبيهة بـ:

```text
src-tauri/src/
├── commands/
│   └── runtime.rs
├── application/
│   └── runtime_status.rs
├── infrastructure/
│   ├── paths.rs
│   └── database/
│       ├── connection.rs
│       ├── migrations.rs
│       └── mod.rs
├── error.rs
└── lib.rs
```

يمكن تغيير الأسماء إذا كان التصميم أوضح، لكن:

- SQL access يبقى في infrastructure.
- Tauri command لا يحتوي migration logic.
- path resolution منفصل عن pure database initializer.
- الاختبارات تستطيع تشغيل initializer على temp directory.

---

## 6. الاختبارات الإلزامية

نفّذ اختبارات حقيقية، لا source-text assertions فقط.

### 6.1 قاعدة جديدة

على temp directory:

- إنشاء المجلدات.
- إنشاء قاعدة جديدة.
- تطبيق 4 migrations.
- وجود 49 جدولًا و25 trigger.
- وجود 6 system roles و22 permissions.
- `foreign_key_check` نظيف.
- `foreign_keys=1`.
- journal mode الفعلي موثق.

### 6.2 إعادة التشغيل

شغّل initializer مرتين على القاعدة نفسها:

- لا migration مكررة.
- لا seed مكرر.
- data/ledger لا يتغيران بصورة هدامة.
- النتيجة ready في المرتين.

### 6.3 checksum mismatch

بعد initialization، عدّل checksum داخل قاعدة الاختبار ثم شغّل initializer:

- يجب الرفض.
- يجب ذكر version المعنية في internal error.
- لا migration جديدة ولا تعديل بيانات.

### 6.4 newer schema

أضف ledger row مجهولًا في fixture صالح:

- runtime يرفض schema الأحدث.
- لا يحاول downgrade أو reset.

### 6.5 ledger gap/partial state

أنشئ حالات اختبار لسجل غير متصل أو metadata غير مطابقة:

- رفض واضح.
- لا متابعة صامتة.

### 6.6 atomic migration failure

استخدم catalog اختباري injectable يحتوي SQL تفشل بعد كتابة أولية:

- لا يبقى جدول/صف جزئي من migration الفاشلة.
- لا يسجل ledger row.
- لا تطبق التالية.

### 6.7 seed idempotency/atomicity

- تطبيق seed مرتين يبقي الأعداد الصحيحة.
- seed اختبارية فاشلة تتراجع كليًا.

### 6.8 runtime status

اختبر serialization والأسماء `camelCase`، وعدم وجود absolute database path.

### 6.9 Tauri mock runtime

حافظ على نجاح الاختبار الفعلي الحالي على Ubuntu وWindows.

لا تستخدم:

```text
#[ignore]
#[cfg(not(windows))]
[lib] test = false
continue-on-error
```

---

## 7. CI المطلوب

أنشئ فقط:

```text
.github/workflows/runtime-ci.yml
```

يعمل على:

```text
ubuntu-latest
windows-latest
```

ويراقب paths المتعلقة بـPHASE 02 وdatabase source inputs.

يجب أن ينفذ:

```text
python scripts/verify_schema.py
git diff --exit-code a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9 -- database scripts/verify_schema.py
npm ci
npm run typecheck
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features --locked -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets --locked -- --nocapture
npm run desktop:check
git diff --check
git status --short --untracked-files=all
```

أضف guard يثبت عدم تعديل:

```text
src/app
src/components
src/i18n
src/styles
src/features
src/bootstrap
package.json
package-lock.json
vite.config.ts
src/main.tsx
index.html
tsconfig.json
tsconfig.app.json
tsconfig.node.json
AGENTS.md
.gitignore
README.md
docs/spec/POSMAN-Blueprint-v1.md
docs/PHASE-01-REPORT.md
docs/BOOTSTRAP-GATE-02-REPORT.md
docs/architecture/accounting-posting.md
docs/architecture/data-dictionary.md
docs/architecture/database-decisions.md
docs/architecture/erd.md
docs/architecture/migration-policy.md
docs/architecture/parallel-wave-02-contract.md
docs/architecture/desktop-shell.md
.github/workflows/schema-ci.yml
.github/workflows/desktop-bootstrap-ci.yml
```

مقارنةً بالـbaseline المعتمد.

أضف Ubuntu MSRV check حقيقيًا بـRust `1.85`:

```text
cargo check --manifest-path src-tauri/Cargo.toml --all-targets --locked
```

لا تكرر workflow Bootstrap manifest inspection؛ workflow الموجود سيعمل تلقائيًا عند تعديل `src-tauri/**`. يجب أن يظل أخضر على Windows وUbuntu.

احتفظ بمخرجات failure diagnostics خارج worktree. لا تستخدم `continue-on-error`.

---

## 8. التوثيق المطلوب

أنشئ:

```text
docs/architecture/runtime-database.md
docs/architecture/runtime-command-contracts.md
docs/PHASE-02-REPORT.md
```

### runtime-database.md

يوثق:

- data paths.
- connection contract.
- WAL behavior/fallback.
- migration catalog/checksums.
- atomic application algorithm.
- seed policy.
- compatibility/refusal rules.
- startup flow.
- thread/blocking model.
- backup-before-future-migration بوصفه requirement مؤجلًا، لا ميزة منفذة.

### runtime-command-contracts.md

يوثق:

- `get_runtime_status`.
- request/response types.
- error envelope.
- عدم كشف المسار أو SQL.
- أن business commands خارج المرحلة.
- contract الذي سيستهلكه Integration Gate لاحقًا.

### PHASE-02-REPORT.md

لا تكتب PASS قبل وجود evidence حقيقي. يجب أن يتضمن:

- baseline والرأس النهائي.
- الملفات المنشأة/المعدلة/المحذوفة.
- dependencies وأسبابها.
- نتائج كل اختبار.
- نتائج Ubuntu وWindows وروابط Actions.
- migration/seed evidence.
- نتائج حماية ownership.
- المخاطر والميزات المؤجلة.
- تأكيد عدم الدمج.

---

## 9. خارج النطاق حتميًا

لا تنفذ في PHASE 02:

- company setup wizard.
- users/password hashing/login/sessions.
- products/families CRUD.
- customers/suppliers CRUD.
- taxes/pricing services.
- inventory opening/posting.
- CUMP.
- stock reservations.
- sales/purchases workflows.
- document conversion.
- accounting posting.
- PDF/printing.
- backup/restore implementation.
- frontend integration.
- UI أو design system.
- installer/signing/updater.
- network/cloud/telemetry.

لا تنشئ placeholders كبيرة لهذه المجالات. اتركها لمراحلها.

---

## 10. قواعد Git وتقليل الاستهلاك

- ابدأ من baseline الحرفي فقط.
- استخدم فرعًا واحدًا وDraft PR واحدًا.
- لا تستخدم force-push أو rebase أو history rewrite.
- لا تنشئ helper workflows أو materializer files لنقل الكود.
- لا تنشئ commits خاصة بنشر commit log.
- اجمع التعديلات المتماسكة؛ استهدف 3 إلى 6 commits مفهومة بدل commit لكل سطر.
- شغّل الفحوص محليًا قبل الدفع متى كانت البيئة تسمح.
- لا تكرر CI عشوائيًا؛ افحص السبب ثم أصلح دفعة متماسكة.
- لا تعدّل ملفات غير مرتبطة لتنظيفها.
- لا تدمج PR.

Commits مقترحة:

```text
feat(runtime): add embedded SQLite initialization
test(runtime): verify migrations and compatibility guards
ci(runtime): validate local runtime on Windows and Ubuntu
docs(runtime): document Phase 02 foundation
```

---

## 11. شروط الإيقاف

توقف واطلب قرارًا إذا:

- تغير `main` عن baseline قبل إنشاء الفرع.
- احتجت تعديل migration مقبولة.
- احتجت ملفًا frozen أو مملوكًا لـPHASE 03.
- تعذر الحفاظ على Windows manifest fix.
- dependency أساسية لا تدعم Rust 1.85.
- الاختبار الحقيقي يفشل على Windows.
- لا يمكن إثبات atomicity/checksum compatibility.
- احتجت توسيع النطاق إلى business logic.

لا تقدّم workaround يضعف الاختبارات أو المعمارية.

---

## 12. معايير القبول النهائية

PHASE 02 مقبولة فقط إذا:

- قاعدة جديدة تُنشأ تلقائيًا دون تثبيت خارجي.
- migrations الأربع وseed تُطبقان بأمان.
- إعادة التشغيل idempotent.
- checksum mismatch/newer schema/gap تُرفض.
- failure ذري ولا يترك schema جزئية.
- PRAGMA contract مثبت بالاختبارات.
- real Tauri mock test ينجح على Windows وUbuntu.
- Desktop Bootstrap CI يبقى أخضر.
- PHASE 01 verifier يبقى أخضر.
- لا ملف خارج ownership تغيّر.
- Draft PR مفتوح وغير مدمج.

---

## 13. صيغة التسليم الإلزامية

قدّم تقريرًا عربيًا واضحًا يحتوي:

1. Repository.
2. Branch.
3. Final head SHA.
4. Draft PR URL وحالته.
5. commits.
6. created/modified/deleted files.
7. architecture implemented.
8. dependency changes.
9. migration catalog SHA evidence.
10. test commands والمخرجات الفعلية المختصرة.
11. GitHub Actions runs ونتيجة كل OS/job.
12. ownership/frozen-file diff evidence.
13. local validation المنفذ فعليًا وما لم يُنفذ.
14. risks/deferred scope.
15. تأكيدات:
    - no force-push.
    - no merge.
    - PHASE 03 لم تُنفذ.
    - business phases لم تبدأ.

لا تقل إن المرحلة مكتملة إذا كان أي check إلزامي pending أو failed.
