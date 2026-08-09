# POSMAN Windows code-signing policy

## Repository state

The repository intentionally contains no PFX/P12 file, private key, certificate
password, signing token, certificate thumbprint, or timestamp credential. CI can
therefore build and validate an unsigned release candidate without exposing a
secret.

## Production publishing gate

Before selling or broadly publishing `POSMAN-Setup-Offline.exe`, the release
owner should obtain a Windows Authenticode code-signing certificate from a
trusted provider and sign both the application executable and final installer
in a controlled external release environment.

Required production checks:

1. Import or access the certificate only in the protected signing environment.
2. Sign with SHA-256 and an RFC 3161 timestamp provided by the certificate vendor.
3. Run `Get-AuthenticodeSignature` and require `Status = Valid`.
4. Recompute and publish `SHA256SUMS.txt` after signing.
5. Install the signed file on a clean Windows 10/11 machine and repeat the
   upgrade/uninstall data-retention test.
6. Remove ephemeral certificate access after the release.

Never commit signing material, echo it to CI logs, put it in an artifact, or
share it through a public issue or pull request. Certificate-provider details
remain an operational decision because no vendor credential has been supplied.

An unsigned GitHub Actions artifact is evidence for engineering validation,
not the final public sales artifact. It may trigger Microsoft SmartScreen.
