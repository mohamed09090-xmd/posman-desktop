# تثبيت POSMAN على Windows / Installation de POSMAN

## العربية

### التثبيت

1. تحقّق من قيمة SHA-256 المنشورة بجانب `POSMAN-Setup-Offline.exe`.
2. انقر بزر الفأرة الأيمن على الملف واختر **تشغيل كمسؤول**.
3. اختر العربية أو الفرنسية أو الإنجليزية وأكمل التثبيت.
4. شغّل POSMAN من سطح المكتب أو قائمة Start.
5. أكمل إعداد الشركة والمستخدم الإداري، ثم أنشئ نسخة احتياطية أولى.

البرنامج يُثبت تحت `C:\Program Files\POSMAN`، بينما تحفظ بيانات العمل تحت:

```text
%LOCALAPPDATA%\POSMAN
```

ويتضمن هذا المسار قاعدة البيانات والنسخ الاحتياطية والوثائق والقوالب والسجلات.

### التحديث

أغلق POSMAN، ثم شغّل المثبّت الأحدث كمسؤول. يمنع المثبّت تثبيت إصدار أقدم
فوق إصدار أحدث. لا تنقل أو تحذف مجلد `%LOCALAPPDATA%\POSMAN` أثناء التحديث.
من الأفضل إنشاء نسخة احتياطية موثقة قبل أي تحديث تشغيلي.

### الإزالة

يمكن إزالة البرنامج من Windows Settings أو من `uninstall.exe`. الإزالة تحذف
ملفات البرنامج والاختصارات فقط، وتُبقي بيانات المتجر تحت
`%LOCALAPPDATA%\POSMAN`. حذف بيانات المتجر قرار منفصل وخطير ويجب ألا يُنفذ
إلا بعد نسخ احتياطية والتحقق منها وموافقة صاحب المتجر صراحةً.

## Français

### Installation

1. Vérifiez le SHA-256 publié de `POSMAN-Setup-Offline.exe`.
2. Exécutez l'installateur en tant qu'administrateur.
3. Choisissez la langue puis terminez l'installation.
4. Lancez POSMAN depuis le Bureau ou le menu Démarrer.
5. Configurez la société et le compte administrateur, puis créez une première sauvegarde.

L'application est installée dans `C:\Program Files\POSMAN`. Les données métier
restent dans `%LOCALAPPDATA%\POSMAN`.

### Mise à niveau et désinstallation

Fermez POSMAN et lancez le nouvel installateur en tant qu'administrateur. Une
version antérieure ne peut pas remplacer une version plus récente. La
désinstallation supprime l'application et ses raccourcis mais conserve les
données locales. La suppression de ces données doit être une opération séparée,
explicite et précédée d'une sauvegarde vérifiée.

## Dépannage / استكشاف الأخطاء

- No internet is required by the installer; WebView2 is embedded.
- If Windows SmartScreen warns about an unsigned build, compare its SHA-256
  with the trusted release manifest and contact the vendor. Production sales
  should use a signed installer.
- Keep at least one verified backup outside the computer before maintenance.
