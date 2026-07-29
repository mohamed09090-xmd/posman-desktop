# تقرير تنفيذ PHASE 02 — Local Runtime Foundation

## 1. الحالة وهوية التسليم

هذا تقرير evidence للتنفيذ، وليس اعتمادًا ذاتيًا للمرحلة. قرار القبول أو الرفض يبقى للمعماري/المراجع الخارجي.

- Repository: `https://github.com/mohamed09090-xmd/posman-desktop`
- Baseline المعتمد: `a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9`
- Validated implementation head: `243d8b131e0965572f60df0354c2f848c78d0e0a`
- Branch: `phase/02-runtime-foundation`
- Draft Pull Request: `https://github.com/mohamed09090-xmd/posman-desktop/pull/3`
- عنوان PR: `[Phase 02] POSMAN local runtime foundation`
- حالة PR وقت إعداد التقرير: Open / Draft / غير مدمج / auto-merge غير مفعّل.
- ملاحظة SHA: commit هذا التقرير توثيقي فقط ويصبح رأس التسليم اللاحق للرأس المتحقق أعلاه؛ رأس الفرع الفعلي يُقرأ من Git/PR ويُذكر في رسالة التسليم النهائية.

## 2. الملفات

### ملفات أُنشئت

```text
.github/workflows/runtime-ci.yml
docs/architecture/runtime-command-contracts.md
docs/architecture/runtime-database.md
docs/PHASE-02-REPORT.md
src-tauri/src/application/mod.rs
src-tauri/src/application/runtime_status.rs
src-tauri/src/commands/mod.rs
src-tauri/src/commands/runtime.rs
src-tauri/src/error.rs
src-tauri/src/infrastructure/mod.rs
src-tauri/src/infrastructure/paths.rs
src-tauri/src/infrastructure/database/connection.rs
src-tauri/src/infrastructure/database/migrations.rs
src-tauri/src/infrastructure/database/mod.rs
src-tauri/src/infrastructure/database/tests.rs
```

### ملفات عُدلت

```text
src-tauri/Cargo.toml
src-tauri/Cargo.lock
src-tauri/src/lib.rs
```

### ملفات حُذفت

لا يوجد.

## 3. التنفيذ المعماري

- استخدام `rusqlite` مباشرةً مع ميزة `bundled`، دون Tauri SQL plugin أو SQLite executable أو sidecar أو server أو Docker.
- حل مجلد البيانات عبر Tauri `local_data_dir()` ثم `POSMAN`، وإنشاء `data`, `backups`, `documents`, `templates`, `logs` وقاعدة `data/posman.sqlite3`.
- فصل path adapter الحقيقي عن `RuntimePaths` القابلة للاختبار بواسطة root مؤقت صريح؛ الاختبارات لا تكتب إلى مجلد المستخدم.
- كل اتصال writable يطبق ويتحقق من `PRAGMA foreign_keys = ON` و`PRAGMA busy_timeout = 5000` قبل transaction، ويطلب WAL ويحتفظ بالقيمة الفعلية التي أعادتها SQLite.
- تضمين migrations الأربع وseed المرجعي من ملفات `database/**` الأصلية وقت البناء؛ لم تُنسخ SQL إلى `src-tauri` ولم تُنشأ migration خامسة.
- فرض catalog متصل، وledger prefix متصل ومتطابق في `id`, `version`, `name`, `checksum_sha256`.
- رفض schema أحدث، checksum mismatch، gap، partial/metadata mismatch كأخطاء fatal دون reset أو downgrade.
- تنفيذ كل migration عبر transaction من نوع `BEGIN IMMEDIATE` بحيث يلتزم SQL وصف ledger معًا أو يتراجعان معًا؛ عند الفشل لا تُنفذ migration التالية.
- تنفيذ seed في transaction مستقلة، idempotent وذرية.
- readiness gate يتحقق من `PRAGMA foreign_key_check`، وجود 49 جدولًا، أربع migrations، وschema version `0004`.
- error model typed يفصل أخطاء المسار، فتح/تهيئة SQLite، schema غير المدعومة، checksum، ledger، migration، seed، والسلامة.
- managed state لا يحتوي `rusqlite::Connection` عالمية؛ يحتفظ بخدمة runtime وحالة readiness فقط.
- تهيئة runtime تتم داخل Tauri `setup` قبل تسجيل state وقبل إتاحة command.
- `get_runtime_status` command للقراءة فقط، يعمل عبر `spawn_blocking`، ويعيد فقط الحقول الخمسة ذات أسماء `camelCase` دون path أو SQL أو بيانات شركة/مستخدم.

## 4. dependencies

- `rusqlite 0.32` مع `bundled`: SQLite محلية مدمجة مباشرة.
- `sha2 0.10`: SHA-256 لملفات migrations بنفس bytes UTF-8 التي يستخدمها verifier.
- `serde 1` مع derive: عقد IPC typed وserialization.
- `serde_json 1` كـdev-dependency: اختبار عقد serialization فقط.

تم قفل graph بإصدارات متوافقة مع Rust `1.85`. لم تُضف ORM، async database framework، Tokio كامل، UUID، Decimal، password hashing، logging stack، network/cloud dependency، أو dependency لمرحلة business.

## 5. migration catalog SHA-256 evidence

```text
0001_system_company_security.sql          af2d8df4e6aadb0333a5b5e7e893d85da0527e4c286462d1fb1c1861fa272735
0002_reference_catalog_partners.sql       f7aab1bb8f8784624cadb4cc9d1cb7e6dde56cad1cbffffa4da90a8e48e7b715
0003_commerce_inventory.sql               093aa71fe7e8ba58b6b487a7c578cd39c353b3225783ce87cabf6a2e8a111d39
0004_accounting_documents_audit.sql       c7d9ac5e194f1c1f47cd4d37f691218635fc6a98b23dd9afbb5a541538f7d99e
```

القيم سُجلت بواسطة Python `hashlib.sha256(path.read_text(encoding="utf-8").encode("utf-8")).hexdigest()` داخل Runtime CI على مصادر database غير المعدلة.

## 6. الاختبارات السلوكية

أضيفت اختبارات حقيقية تغطي:

- قاعدة جديدة، إنشاء المجلدات والملف، 49 جدولًا، 25 trigger، 6 system roles، 22 permissions، PRAGMA contract، و`foreign_key_check` نظيف.
- إعادة initialization على القاعدة نفسها دون تكرار migrations أو seed.
- checksum mismatch مع ذكر version داخليًا ومنع أي متابعة.
- unknown newer schema دون downgrade أو reset.
- ledger gap وmetadata mismatch.
- migration catalog اختباري يفشل بعد كتابة أولية، مع إثبات rollback وعدم تسجيل ledger وعدم تشغيل التالية.
- seed فاشلة تتراجع كليًا، وتطبيق seed مرتين يبقي الأعداد ثابتة.
- `RuntimeStatus` بأسماء camelCase وخمسة حقول فقط ودون absolute path أو SQL.
- اختبار Tauri mock الحقيقي `application_setup_builds_with_mock_runtime`، مع تشغيل setup فعليًا والتحقق من managed state وقاعدة SQLite.

نتيجة `cargo test --manifest-path src-tauri/Cargo.toml --all-targets --locked -- --nocapture`:

- Ubuntu: `10 passed; 0 failed; 0 ignored`.
- Windows: `10 passed; 0 failed; 0 ignored`.

## 7. أوامر التحقق ونتائجها

كل الأوامر التالية نُفذت بنجاح داخل GitHub Actions على Ubuntu وWindows حيث ينطبق:

```text
python scripts/verify_schema.py

git diff --exit-code a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9 -- database scripts/verify_schema.py

git diff --exit-code a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9 -- <frozen/out-of-scope paths>

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

MSRV على Ubuntu نجح:

```text
cargo check --manifest-path src-tauri/Cargo.toml --all-targets --locked
```

باستخدام Rust `1.85`.

## 8. GitHub Actions evidence

### Runtime CI

- Run: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30479216419`
- `Local runtime (ubuntu-latest)`: success.
- `Local runtime (windows-latest)`: success.
- `Rust 1.85 MSRV (ubuntu-latest)`: success.

### Desktop bootstrap CI

- Run: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30479216171`
- `Desktop shell (ubuntu-latest)`: success.
- `Desktop shell (windows-latest)`: success.
- Windows Tauri mock test manifest dependency: success.
- Windows application manifest dependency: success.

### SQLite schema verification

- Run: `https://github.com/mohamed09090-xmd/posman-desktop/actions/runs/30479222072`
- Workflow: success.

## 9. ownership وfrozen-file evidence

Runtime CI قارن بالـbaseline الحرفي ونجح في:

- عدم وجود أي diff داخل `database/**` أو `scripts/verify_schema.py`.
- عدم وجود أي diff داخل UI/PHASE 03 paths أو الملفات المشتركة المجمدة أو وثائق PHASE 01/Bootstrap أو workflows المقبولة.
- `git diff --check` نظيف.
- عدم وجود staged، unstaged، أو untracked diff حقيقي بعد validation؛ تم التعامل على Windows مع line-ending normalization فقط بنفس النمط المعتمد في Desktop Bootstrap CI، دون تجاهل أي diff فعلي.

## 10. التحقق المحلي

لم تُنفذ أوامر Rust/npm baseline أو final محليًا في بيئة الوكيل لأن البيئة لم توفر checkout شبكيًا قابلًا للبناء ولا Rust/Cargo toolchain. لم تُسجل أي نتيجة محلية غير منفذة على أنها ناجحة. التحقق التنفيذي الموثق أعلاه تم عبر GitHub Actions على Ubuntu وWindows، بما في ذلك البناء الأصلي والاختبارات وMSRV.

## 11. المخاطر والنطاق المؤجل

- backup-before-future-migration requirement موثق فقط؛ backup/restore غير منفذ في PHASE 02.
- لا توجد company setup، users/password hashing/login/sessions، CRUD، inventory posting، CUMP، sales/purchases، accounting posting، PDF/printing، installer، updater، أو frontend integration.
- `get_runtime_status` يعرض readiness فقط؛ business commands مؤجلة لمراحلها المعتمدة.
- أي migration مستقبلية تتطلب قرار مرحلة لاحقة ونسخة جديدة مرتبة؛ migrations المقبولة لم تُعدل.

## 12. التأكيدات

- لم يحدث force-push أو rebase أو history rewrite.
- Pull Request بقي Draft ولم يُحوّل إلى Ready.
- Pull Request لم يُدمج ولم يُفعّل auto-merge.
- PHASE 03 لم تبدأ ولم تُعدل ملفاتها.
- لم تبدأ أي مرحلة business أو UI integration.
