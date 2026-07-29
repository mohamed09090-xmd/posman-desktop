# تقرير تنفيذ PHASE 02 — Local Runtime Foundation

## الحالة

هذا تقرير تنفيذ أولي للمرحلة على Draft Pull Request. القبول أو الرفض مسؤولية المعماري/المراجع الخارجي، ولا يمثل هذا المستند اعتمادًا ذاتيًا للمرحلة.

## هوية التسليم

- Repository: `https://github.com/mohamed09090-xmd/posman-desktop`
- Baseline المعتمد: `a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9`
- Branch: `phase/02-runtime-foundation`
- Draft Pull Request: `https://github.com/mohamed09090-xmd/posman-desktop/pull/3`
- عنوان PR: `[Phase 02] POSMAN local runtime foundation`
- الحالة: Open / Draft / غير مدمج
- Final head: سيُثبت بعد اكتمال التحقق النهائي.

## التنفيذ المعماري

- SQLite مدمجة عبر `rusqlite` وميزة `bundled`، دون plugin أو executable أو server خارجي.
- مسارات POSMAN المحلية تُحل عبر Tauri، مع طبقة paths قابلة للاختبار بواسطة root مؤقت صريح.
- كل اتصال يطبق ويتحقق من `foreign_keys=ON` و`busy_timeout=5000`، ويطلب WAL مع تسجيل النمط الفعلي.
- migrations الأربع الحالية وseed المرجعي مضمنة من مصادر `database/**` الأصلية دون نسخ SQL أو إضافة migration.
- ledger يجب أن يكون prefix متصلًا ومتطابقًا في id/version/name/SHA-256؛ schema الأحدث أو checksum/gap/metadata غير المطابقة تمنع startup.
- كل migration ذرية باستخدام transaction فورية، والـseed في transaction مستقلة ذرية.
- readiness يتطلب `foreign_key_check` نظيفًا، 49 جدولًا، أربع migrations، وschema `0004`.
- Tauri state لا يخزن `rusqlite::Connection` عالمية؛ الأمر `get_runtime_status` للقراءة فقط ويعيد خمسة حقول آمنة.

## تغييرات dependencies

- `rusqlite` مع `bundled`: SQLite محلية مدمجة مباشرة.
- `sha2`: SHA-256 متوافق مع verifier المقبول.
- `serde` derive: عقد IPC typed وcamelCase.
- `serde_json` كـdev-dependency: اختبار serialization فقط.

لم تُضف ORM، async database framework، Tokio كامل، UUID، decimal، password hashing، logging stack، cloud dependency، أو business dependency.

## الاختبارات المنفذة

الاختبارات المضافة تغطي قاعدة جديدة، 49 جدولًا و25 trigger، 6 أدوار و22 صلاحية، PRAGMA contract، إعادة التشغيل idempotent، checksum mismatch، newer schema، ledger gap/metadata mismatch، ذرية migration الفاشلة، ذرية/idempotency الـseed، عقد runtime status، واختبار Tauri mock الحقيقي باسم `application_setup_builds_with_mock_runtime`.

نتائج الأوامر وروابط Ubuntu/Windows وMSRV ستُسجل هنا بعد تشغيل final-head workflows. لا توجد مطالبة PASS قبل توفر evidence نهائي.

## حماية الملكية

لم تُعدّل migrations أو seed أو verifier أو ملفات UI/PHASE 03 أو الملفات المشتركة المجمدة. Final Runtime CI يحتوي guards مقارنةً بالـbaseline الحرفي.

## المخاطر والنطاق المؤجل

- backup-before-future-migration requirement موثق فقط؛ backup/restore غير منفذ.
- لا توجد company setup أو users/login أو CRUD أو inventory/accounting workflows أو PDF/printing أو frontend integration.
- صحة Windows النهائية تعتمد على GitHub Actions لأن بيئة التنفيذ المحلية لا تحتوي Windows toolchain أو checkout شبكي قابل للبناء.

## التأكيدات

- لم يحدث force-push أو rebase أو history rewrite.
- Pull Request لم يُدمج ولم يُفعّل auto-merge.
- PHASE 03 لم تبدأ.
- لم تبدأ أي مرحلة business.
