# POSMAN — PHASE 03 Original UI Foundation Execution Pack

## 0. هوية المهمة

أنت وكيل التنفيذ المسؤول حصريًا عن **PHASE 03 — Original UI Foundation** في مشروع POSMAN.

هذه مهمة تصميم UX/UI وتنفيذ React فعلي واختبارات بصرية ووظيفية داخل GitHub. نفّذ ضمن ملكية PHASE 03 فقط، افتح Draft Pull Request، واتركه دون دمج للمراجعة الخارجية.

### المستودع

```text
https://github.com/mohamed09090-xmd/posman-desktop
```

### baseline المعتمد حصريًا

```text
a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9
```

### الفرع المطلوب

```text
phase/03-ui-foundation
```

### Draft PR المطلوب

```text
[Phase 03] POSMAN original UI foundation
```

يستهدف:

```text
main
```

لا تحوّله إلى Ready، ولا تدمجه، ولا تستخدم auto-merge.

---

## 1. Bootstrap Gate قبل أي تعديل

قبل إنشاء الفرع أو تعديل الملفات:

1. تأكد أن `main` يساوي حرفيًا:

   ```text
   a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9
   ```

2. اقرأ كاملًا:

   ```text
   AGENTS.md
   docs/spec/POSMAN-Blueprint-v1.md
   docs/architecture/parallel-wave-02-contract.md
   docs/architecture/desktop-shell.md
   docs/BOOTSTRAP-GATE-02-REPORT.md
   ```

3. افحص كامل frontend الحالي:

   ```text
   package.json
   package-lock.json
   vite.config.ts
   src/main.tsx
   src/app/AppRoot.tsx
   src/bootstrap/bootstrap.css
   index.html
   ```

4. شغّل baseline checks المتاحة:

   ```text
   python scripts/verify_schema.py
   npm ci
   npm run typecheck
   npm run build
   npm run desktop:check
   ```

5. راجع واجهة Bootstrap فعليًا إن كانت البيئة تسمح، وسجّل مشاكلها دون اعتبارها design foundation.

إذا لم يطابق `main` baseline، أو كان الفرع موجودًا برأس غير متوقع، أو فشل baseline قبل تعديلاتك، توقف وقدّم blocker report.

---

## 2. الهدف الدقيق للمرحلة

إنشاء لغة واجهة أصلية وعملية لـPOSMAN باسم:

```text
دفتر العمليات المعاصر
Contemporary Operations Ledger
```

وتنفيذها كـReact UI gallery قابلة للتشغيل داخل Tauri، عربية افتراضيًا مع French/LTR، وتحتوي الشاشات والمكونات المرجعية التي ستُبنى عليها المراحل التشغيلية.

المطلوب ليس dashboard تجميليًا ولا business app كاملًا. المطلوب:

- design tokens.
- application shell.
- navigation model.
- i18n/direction foundation.
- reusable operational components.
- representative screens ببيانات fixtures واضحة.
- accessibility/responsive/visual evidence.

لا يوجد backend integration في هذه المرحلة.

---

## 3. حدود الملكية الإلزامية

### الملفات التي تملكها PHASE 03

يجوز إنشاء أو تعديل:

```text
src/app/**
src/components/**
src/i18n/**
src/styles/**
src/features/ui-gallery/**
frontend test files
public/**
package.json
package-lock.json
vite.config.ts
docs/design/**
.github/workflows/ui-ci.yml
docs/PHASE-03-REPORT.md
```

### ممنوع تعديلها

```text
src-tauri/**
database/**
scripts/verify_schema.py
src/platform/tauri/**
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

### ملاحظة مهمة عن CSS

لا تعدل:

```text
src/bootstrap/bootstrap.css
```

لأنه خارج ملكية PHASE 03. استورد stylesheet الجديد من:

```text
src/app/AppRoot.tsx
```

بحيث تأتي styles الجديدة بعد bootstrap CSS وتستبدل مظهره دون تغيير الملف المشترك.

إذا احتجت ملفًا خارج الملكية، توقف واطلب Integration Patch Decision قبل التعديل.

---

## 4. اتجاه التصميم المعتمد

اتجاه المنتج محسوم في Blueprint؛ لا تطلب من المستخدم إعادة اختيار palette أو layout.

قبل التنفيذ أنشئ دراسة مختصرة:

```text
docs/design/direction-study.md
```

تتضمن ثلاثة اتجاهات مختلفة فعلًا:

1. دفتر العمليات المعاصر — الاتجاه المعتمد.
2. اتجاه بديل عالي الكثافة.
3. اتجاه بديل مبسط للمبتدئ.

قارن:

- composition.
- density.
- navigation.
- typography.
- interaction model.
- accessibility.
- maintenance cost.

ثم وثق لماذا الاتجاه الأول هو المنفذ وفق Blueprint. لا تنتظر موافقة جديدة؛ القرار معتمد مسبقًا.

---

## 5. قواعد منع القالب العام

الواجهة غير مقبولة إذا بدت كقالب Admin أو SaaS مولد آليًا.

ممنوع:

- sidebar تقليدي من قالب admin.
- KPI cards متكررة.
- bento grid.
- glassmorphism.
- gradients زرقاء/بنفسجية عامة.
- حواف دائرية كبيرة.
- floating cards في كل مكان.
- hero section.
- blobs أو 3D أو stock photos.
- shadows ثقيلة.
- default component-library appearance.
- animation للزينة.
- نسخ Sage أو أي منتج معروف بصريًا.

الأصالة تأتي من منطق المنتج:

- صفحات تشبه سجل عمليات منظمًا لا موقع SaaS.
- خطوط وفواصل دقيقة.
- مساحات هادئة مع كثافة مدروسة.
- حالات المستند كأختام تشغيلية.
- تسلسل المستندات كشريط عملية.
- إجراءات ثابتة وقريبة من سياق العمل.
- جدول بيانات واضح هو البطل، لا بطاقات زخرفية.

---

## 6. النظام البصري الإلزامي

ابدأ من tokens المعتمدة:

| Token | القيمة |
|---|---|
| App background | `#F4F0E7` |
| Document surface | `#FFFCF6` |
| Primary text | `#1E2523` |
| Secondary text | `#66706C` |
| Borders | `#D8D0C2` |
| Confirmed | `#1F5A45` |
| Pending | `#B7791F` |
| Error/shortage | `#B74A3C` |
| Radius | `4px` إلى `6px` |
| Spacing | `4 / 8 / 12 / 16 / 24 / 32` |

أنشئ CSS custom properties منظمة لـ:

- colors and surfaces.
- typography.
- spacing.
- borders/radii.
- shadows المحدودة.
- focus.
- motion 120–180ms.
- z-index layers.
- data density.

لا تكرر قيمًا عشوائية في المكونات.

### الخط

الأولوية:

```text
IBM Plex Sans Arabic
IBM Plex Sans
```

يجب أن تعمل الخطوط offline:

- استخدم ملفات WOFF2 محلية من المصدر الرسمي المرخص.
- أضف ملف الترخيص المناسب داخل `public/fonts/`.
- حمّل الأوزان اللازمة فقط لتقليل الحجم.
- لا تستخدم Google Fonts أو CDN أو runtime request.
- إذا تعذر الحصول الموثوق على الملفات الرسمية، استخدم system fallback مؤقتًا ووثق ذلك كـblocker بصري؛ لا تضف ملفات مجهولة.

---

## 7. i18n واتجاه الصفحة

أنشئ foundation خفيفًا ومكتوب الأنواع يدعم:

```text
ar-DZ — default — RTL
fr-DZ — LTR
```

متطلبات:

- كل النصوص المرئية في dictionaries، لا strings موزعة عشوائيًا.
- language switch يعمل دون reload.
- تحديث `document.documentElement.lang`.
- تحديث `document.documentElement.dir`.
- استخدام CSS logical properties.
- لا تبنِ layout منفصلًا للعربية والفرنسية.
- الأيقونات الاتجاهية فقط تنعكس عندما يتغير معناها.
- تنسيق DZD والأرقام والتواريخ عبر `Intl`.
- لا تترك مفاتيح ناقصة أو خليطًا غير مقصود بين اللغتين.
- أسماء المكونات والكود بالإنجليزية؛ microcopy بالعربية والفرنسية.

لا تضف i18n framework كبيرًا إذا كان typed React context صغيرًا أوضح وأسهل.

---

## 8. Application Shell الأصلي

نفّذ داخل `AppRoot`:

### 8.1 Command Bar

يحتوي:

- هوية POSMAN مختصرة.
- اسم مساحة العمل الحالية.
- بحث شامل شكلي بوضوح أنه UI fixture.
- زر «إنشاء» تتغير تسميته حسب مساحة العمل.
- اللغة.
- اسم المؤسسة التجريبي.
- حالة محلية/offline.

لا يشبه top navbar لموقع ويب.

### 8.2 Workspace Rail

المساحات:

1. اليوم.
2. المبيعات.
3. المشتريات.
4. المخزون.
5. المحاسبة.
6. التقارير.
7. الإدارة.

يجب أن يكون rail مميزًا شبيهًا بفهرس دفتر عمليات، وليس sidebar بقائمة icon+label داخل مستطيلات مستديرة.

### 8.3 Workspace

- semantic landmarks.
- skip link.
- header واضح.
- content يحافظ على الموضع.
- internal scroll مدروس.
- لا تجعل document كله سلسلة cards.

### 8.4 Navigation

هذه UI gallery؛ استخدم React state أو حلًا خفيفًا داخل `AppRoot`.

لا تضف router كبيرًا إلا إذا أثبتت الحاجة. لا تعدل `src/main.tsx`.

---

## 9. المكونات المرجعية الإلزامية

أنشئ مكونات reusable حقيقية:

```text
CommandBar
WorkspaceRail
WorkspaceHeader
DocumentCanvas
ProcessStrip
StatusStamp
DataGrid
DetailDrawer
ActionDock
Field / Input / Select primitives
Button variants
InlineNotice
EmptyState
LoadingState
```

### قواعد المكونات

- semantic HTML.
- props typed.
- حالات default/hover/focus/active/disabled/error/loading.
- visible focus.
- لا تعتمد الحالة على اللون وحده.
- touch/click targets مناسبة.
- لا abstraction مبكرًا دون reuse.
- لا component file ضخم يضم التطبيق كله.
- لا component library افتراضية.

يمكن استخدام icon family صغيرة مثل `lucide-react` فقط إذا:

- السبب موثق.
- الاستيراد tree-shakable.
- الأسلوب موحد ومخصص.
- لا تستبدل labels بأيقونات غامضة.

لا تضف animation library؛ استخدم CSS.

---

## 10. الشاشات المرجعية المطلوبة

استخدم fixtures مكتوبة الأنواع داخل:

```text
src/features/ui-gallery/fixtures/**
```

ويجب أن يكون واضحًا في الكود والتوثيق أنها demo data وليست بيانات عميل.

### 10.1 «اليوم»

ليست dashboard KPI.

تعرض:

- الأعمال التي تحتاج تدخلًا.
- طلبات جاهزة للتسليم.
- تسليمات جاهزة للفوترة.
- فواتير غير مرحلة.
- مواد عند الحد الأدنى.
- آخر العمليات.
- shortcuts لبيع/شراء/جرد.

استخدم قائمة عمليات/ledger، لا شبكة بطاقات.

### 10.2 قائمة المواد

تعرض:

- code/barcode.
- الاسم.
- العائلة.
- سعر البيع.
- on hand/reserved/available.
- minimum stock.
- status.

تشمل:

- بحث وتصفية UI-only.
- selection واضح.
- empty state.
- فتح `DetailDrawer`.
- table scrolling داخل workspace في الحد الأدنى للنافذة.

### 10.3 بطاقة مادة في Detail Drawer

تعرض sections موجزة:

- التعريف.
- التسعير.
- الضريبة.
- المخزون.
- آخر حركة.

لا تنفذ حفظًا حقيقيًا.

### 10.4 المخزون الافتتاحي

واجهة مستند تتضمن:

- warehouse/date header.
- lines table.
- quantity/cost fields.
- totals.
- warning أن الترحيل سينشئ حركات.
- Action Dock.

كل الأزرار UI demo فقط، دون كتابة أو Tauri invoke.

### 10.5 فاتورة بيع

تستخدم:

- Document Canvas.
- Status Stamp.
- header fields.
- line grid.
- line/general discounts.
- HT/TVA/TTC totals.
- Action Dock.
- validation notice.

الأرقام تعرض بـDZD وfixtures ثابتة. لا تنفذ حساب business نهائي؛ اكتب fixtures ذات totals متسقة واختبرها.

### 10.6 دورة البيع

استخدم `ProcessStrip` لعرض:

```text
طلب عميل → سند تسليم → فاتورة → محاسبة
```

اعرض:

- completed/current/pending.
- partial delivery example `8 + 12 من 20`.
- status نصي وأيقوني، لا لون فقط.

### 10.7 معرض الحالات

صفحة داخلية صغيرة للمراجعة تعرض:

- الأزرار.
- الحقول.
- status stamps.
- notices.
- empty/loading/error states.
- density examples.

لا تحولها إلى Storybook كامل.

---

## 11. Responsive وWindow Constraints

POSMAN تطبيق Windows Desktop، وليس تطبيق هاتف.

تحقق على الأقل:

```text
1024 × 640  — الحد الأدنى المعتمد
1280 × 800  — الحجم الافتراضي
1440 × 900
1600 × 900
```

متطلبات:

- لا page-level horizontal overflow.
- الجداول الطويلة تستخدم internal scroll.
- Command Bar لا يقطع الإجراءات.
- rail يبقى مفهومًا عند 1024px.
- Action Dock لا يحجب المحتوى.
- النص الفرنسي الأطول لا يكسر layout.
- دعم zoom/text scaling حتى 200% بصورة عملية.
- لا تفترض hover فقط.

لا تضيع الوقت على mobile layout خارج هدف المنتج، لكن لا تجعل CSS ينهار تحت 1024px.

---

## 12. Accessibility

استهدف نتائج عملية بمستوى AA:

- landmarks صحيحة.
- heading hierarchy.
- skip link.
- keyboard navigation.
- focus order.
- focus visible.
- contrast.
- status غير معتمد على اللون.
- labels مرتبطة بالحقول.
- أسماء screen reader للأيقونات والأزرار.
- عدم وجود hover-only content.
- `prefers-reduced-motion`.
- reduced motion يلغي transitions غير الضرورية.
- drawer غير modal أو focus management صحيح إن كان modal.
- live announcements فقط عند الحاجة.

لا تضف ARIA إذا كان HTML semantic يكفي.

---

## 13. الأداء

استهدف أجهزة 4GB RAM:

- لا صور زخرفية.
- لا WebGL.
- لا فيديو.
- لا chart library.
- لا animation framework.
- لا state-management framework كبير.
- لا تحميل خطوط أو assets من الشبكة.
- لا بيانات ضخمة داخل bundle.
- imports قابلة للتقسيم عند الحاجة.

DataGrid في هذه المرحلة تستخدم fixture محدودة؛ وثّق virtualization كمتطلب للمراحل التي تعرض بيانات كبيرة ولا تضف مكتبة ضخمة الآن دون حاجة.

---

## 14. Tests وVisual Evidence

يجوز تحديث frontend dependencies وlockfile.

أضف أقل test stack مناسب، مثل:

- Vitest.
- Testing Library.
- user-event.
- axe integration.
- Playwright Chromium للـbrowser evidence.

### Unit/component tests

اختبر:

- Arabic default.
- تبديل اللغة يغير `lang/dir`.
- اكتمال مفاتيح الترجمة.
- Workspace navigation.
- Product filter وdrawer.
- DZD/date formatting.
- buttons/fields semantics.
- fixture invoice totals consistency.
- reduced-motion CSS/contract قدر الإمكان.

### Browser tests

اختبر:

- لا console errors.
- التنقل بالـrail.
- language switch.
- keyboard focus path.
- product drawer.
- invoice/cycle screens.
- عدم وجود overflow على 1024×640.
- axe: لا critical أو serious violations في الشاشات الأساسية.

### Screenshots

أنشئ screenshots فعلية كـCI artifacts، على الأقل:

1. Arabic Today — 1280×800.
2. French Today — 1280×800.
3. Arabic Invoice — 1024×640.
4. Product list + drawer — 1440×900.
5. Sales Process Strip — 1280×800.

لا تحفظ generated screenshots داخل worktree أثناء CI. استخدم runner temp/artifact directory.

لا تستخدم screenshots بدل الاختبارات.

---

## 15. UI CI

أنشئ:

```text
.github/workflows/ui-ci.yml
```

UI-specific browser job يمكن أن يعمل على Ubuntu، بينما workflow Bootstrap الحالي سيعيد إثبات Tauri build على Windows وUbuntu تلقائيًا بسبب تغييرات `src/**` و`package*.json`.

نفّذ في UI CI:

```text
python scripts/verify_schema.py
git diff --exit-code a4165e28fb3bf8693d8023742e2ac2e7cc5db7d9 -- src-tauri database scripts/verify_schema.py src/platform/tauri src/main.tsx index.html tsconfig.json tsconfig.app.json tsconfig.node.json AGENTS.md .gitignore src/bootstrap/bootstrap.css README.md docs/spec/POSMAN-Blueprint-v1.md docs/PHASE-01-REPORT.md docs/BOOTSTRAP-GATE-02-REPORT.md docs/architecture/accounting-posting.md docs/architecture/data-dictionary.md docs/architecture/database-decisions.md docs/architecture/erd.md docs/architecture/migration-policy.md docs/architecture/parallel-wave-02-contract.md docs/architecture/desktop-shell.md .github/workflows/schema-ci.yml .github/workflows/desktop-bootstrap-ci.yml
npm ci
npm run typecheck
npm run build
npm run test:ui
npm run test:e2e
git diff --check
git status --short --untracked-files=all
```

أضف scan يفشل إذا وجد runtime network clients أو external assets داخل frontend:

```text
fetch(
XMLHttpRequest
WebSocket(
http://
https://
```

مع استثناءات test/tooling موثقة فقط إن لزم، وليس استثناءً عامًا يخفي الشبكة.

ارفع screenshots ونتائج accessibility كـartifacts.

احتفظ بالـartifacts المؤقتة خارج repository. لا تستخدم:

```text
continue-on-error
|| true
```

للفحوص المطلوبة.

---

## 16. التوثيق المطلوب

أنشئ:

```text
docs/design/direction-study.md
docs/design/ui-foundation.md
docs/design/component-inventory.md
docs/PHASE-03-REPORT.md
```

### direction-study.md

- الاتجاهات الثلاثة.
- الفروق الجوهرية.
- سبب اعتماد Contemporary Operations Ledger.
- anti-copy statement.

### ui-foundation.md

- tokens.
- typography/font licensing.
- layout anatomy.
- RTL/LTR strategy.
- density.
- motion.
- accessibility.
- responsive/window behavior.
- fixture-only boundary.

### component-inventory.md

لكل component:

- purpose.
- variants.
- states.
- accessibility contract.
- future business integration point.

### PHASE-03-REPORT.md

لا تدّع PASS قبل evidence. يتضمن:

- baseline والرأس النهائي.
- selected direction.
- created/modified/deleted files.
- dependencies وأسبابها.
- screenshots/artifact links.
- tests/CI results.
- Windows وUbuntu Bootstrap CI.
- ownership guard.
- accessibility findings.
- remaining risks.
- no merge confirmation.

---

## 17. خارج النطاق حتميًا

لا تنفذ:

- Tauri `invoke`.
- SQLite access.
- API/service layer.
- runtime command adapters.
- company setup persistence.
- authentication.
- CRUD حقيقي.
- stock calculations.
- CUMP.
- document transformation logic.
- accounting posting.
- PDF/printing.
- backup/restore.
- installer.
- charts/reports engine.
- cloud/network/telemetry.

لا تعرض زرًا على أنه حفظ بيانات حقيقي. يمكن أن يعطي feedback تجريبيًا واضحًا داخل UI gallery فقط.

---

## 18. قواعد Git وتقليل الاستهلاك

- فرع واحد وDraft PR واحد.
- baseline الحرفي فقط.
- لا force-push، rebase، history rewrite أو auto-merge.
- لا helper workflows أو commits لنقل الملفات.
- لا commit-log helper.
- استهدف 4 إلى 7 commits متماسكة.
- اختبر محليًا قبل الدفع متى أمكن.
- لا تكرر CI قبل فهم الفشل.
- لا تنظف ملفات خارج المهمة.
- لا تدمج PR.

Commits مقترحة:

```text
docs(ui): define POSMAN operations-ledger direction
feat(ui): add bilingual application shell and tokens
feat(ui): build operational component gallery
test(ui): verify accessibility and desktop layouts
ci(ui): validate the UI foundation
docs(ui): record Phase 03 evidence
```

---

## 19. شروط الإيقاف

توقف واطلب قرارًا إذا:

- تغير baseline.
- احتجت تعديل `src/main.tsx` أو `index.html`.
- احتجت تعديل `src-tauri/**` أو `src/platform/tauri/**`.
- احتجت network runtime.
- تعذر الحفاظ على Arabic default وFrench completeness.
- لم تستطع تقديم visual evidence حقيقي.
- Bootstrap CI فشل بسبب تغييرات PHASE 03.
- الواجهة لا تعمل عند 1024×640.
- font source/licensing غير موثوق.

لا تخفِ فشلًا بصريًا أو accessibility violation.

---

## 20. معايير القبول النهائية

PHASE 03 مقبولة فقط إذا:

- الواجهة مميزة بوضوح وغير شبيهة بقالب admin عام.
- Arabic/RTL وFrench/LTR يعملان فعليًا.
- جميع المكونات والشاشات المرجعية المطلوبة أعلاه موجودة.
- التصميم مرتبط بدورة العمل التجارية، لا بالزينة.
- keyboard/focus/contrast/axe checks ناجحة.
- 1024×640 و1280×800 ناجحان دون overflow رئيسي.
- screenshots الفعلية متاحة.
- typecheck/build/tests خضراء.
- Desktop Bootstrap CI أخضر على Windows وUbuntu.
- PHASE 01 verifier أخضر.
- لا ملف خارج ownership تغيّر.
- Draft PR مفتوح وغير مدمج.

---

## 21. صيغة التسليم الإلزامية

قدّم تقريرًا عربيًا واضحًا يحتوي:

1. Repository.
2. Branch.
3. Final head SHA.
4. Draft PR URL وحالته.
5. commits.
6. created/modified/deleted files.
7. design direction ولماذا هو أصلي.
8. component inventory.
9. screens implemented.
10. i18n/RTL/LTR evidence.
11. accessibility evidence.
12. viewport/overflow evidence.
13. screenshot artifact links.
14. dependency changes.
15. test commands والمخرجات الفعلية.
16. GitHub Actions runs لكل workflow/platform.
17. ownership/frozen-file diff evidence.
18. local checks التي نُفذت وما لم يُنفذ.
19. risks/deferred scope.
20. تأكيدات:
    - no force-push.
    - no merge.
    - PHASE 02 لم تُنفذ.
    - no backend/runtime integration.

لا تقل إن المرحلة مكتملة إذا كان أي check أو screenshot أو Windows/Ubuntu validation pending أو failed.
