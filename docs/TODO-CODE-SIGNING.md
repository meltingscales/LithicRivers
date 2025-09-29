# Code Signing Implementation Guide

## Overview
This document outlines the implementation of digital signing for LithicRivers executables across Windows, macOS, and Linux platforms using our existing CI/CD pipeline.

## Platform Requirements

### Windows Code Signing
- **Certificate**: Code signing certificate from trusted CA (DigiCert, GlobalSign, etc.)
- **Cost**: $200-400/year
- **Storage**: Certificate as `.p12/.pfx` file
- **GitHub Secrets**:
  - `WINDOWS_CERTIFICATE_BASE64` (base64 encoded .pfx file)
  - `WINDOWS_CERTIFICATE_PASSWORD`

### macOS Code Signing & Notarization
- **Account**: Apple Developer Program membership ($99/year)
- **Certificate**: Apple Developer ID Application certificate
- **Requirements**: App-specific password for notarization
- **GitHub Secrets**:
  - `MACOS_CERTIFICATE_BASE64`
  - `MACOS_CERTIFICATE_PASSWORD`
  - `MACOS_NOTARIZATION_APPLE_ID`
  - `MACOS_NOTARIZATION_TEAM_ID`
  - `MACOS_NOTARIZATION_PASSWORD` (app-specific password)

### Linux Signing
- **Method**: GPG signing (most common and universal)
- **Cost**: Free
- **Setup**: Generate GPG key pair, export public key for users
- **GitHub Secrets**:
  - `LINUX_GPG_PRIVATE_KEY`

## CI/CD Implementation

### Windows Build Job Additions
```yaml
- name: Import Windows certificate
  if: matrix.os == 'windows-latest'
  run: |
    echo "${{ secrets.WINDOWS_CERTIFICATE_BASE64 }}" | base64 --decode > cert.pfx
    Import-PfxCertificate -FilePath cert.pfx -CertStoreLocation Cert:\CurrentUser\My -Password (ConvertTo-SecureString -String "${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}" -AsPlainText -Force)

- name: Sign Windows executable
  if: matrix.os == 'windows-latest'
  run: |
    signtool sign /fd SHA256 /tr http://timestamp.sectigo.com /td SHA256 /a artifacts\*.exe
```

### macOS Build Job Additions
```yaml
- name: Import macOS certificate
  if: matrix.os == 'macos-latest'
  run: |
    echo "${{ secrets.MACOS_CERTIFICATE_BASE64 }}" | base64 --decode > cert.p12
    security create-keychain -p temp build.keychain
    security default-keychain -s build.keychain
    security unlock-keychain -p temp build.keychain
    security import cert.p12 -k build.keychain -P "${{ secrets.MACOS_CERTIFICATE_PASSWORD }}" -T /usr/bin/codesign

- name: Sign macOS executable
  if: matrix.os == 'macos-latest'
  run: |
    codesign --force --sign "Developer ID Application" --options runtime artifacts/lithicrivers

- name: Notarize macOS executable
  if: matrix.os == 'macos-latest'
  run: |
    ditto -c -k artifacts/lithicrivers lithicrivers.zip
    xcrun notarytool submit lithicrivers.zip --apple-id "${{ secrets.MACOS_NOTARIZATION_APPLE_ID }}" --team-id "${{ secrets.MACOS_NOTARIZATION_TEAM_ID }}" --password "${{ secrets.MACOS_NOTARIZATION_PASSWORD }}" --wait
```

### Linux Build Job Additions
```yaml
- name: Import GPG key and sign Linux executable
  if: matrix.os == 'ubuntu-latest'
  run: |
    echo "${{ secrets.LINUX_GPG_PRIVATE_KEY }}" | gpg --import
    gpg --armor --detach-sign artifacts/lithicrivers
```

## Setup Steps

### Windows
1. Purchase code signing certificate from trusted CA
2. Export certificate as `.pfx` file
3. Convert to base64: `base64 -i cert.pfx`
4. Add to GitHub secrets

### macOS
1. Enroll in Apple Developer Program
2. Create Developer ID Application certificate in Xcode/Developer Portal
3. Export as `.p12` file
4. Create app-specific password in Apple ID settings
5. Add all credentials to GitHub secrets

### Linux
1. Generate GPG key: `gpg --gen-key`
2. Export private key: `gpg --armor --export-secret-keys your@email.com`
3. Export public key for distribution: `gpg --armor --export your@email.com`
4. Add private key to GitHub secrets

## Cost Summary
- **Windows**: $200-400/year (code signing certificate)
- **macOS**: $99/year (Apple Developer Program)
- **Linux**: Free (GPG signing)
- **Total**: ~$300-500/year

## Benefits
- Eliminates security warnings for users
- Builds trust and credibility
- Required for many distribution platforms
- Prevents tampering detection
- Professional software distribution standard

## User Verification

### Windows
Automatic - Windows will show publisher information and no security warnings

### macOS
Automatic - Gatekeeper will allow execution without warnings

### Linux
Users can verify with:
```bash
gpg --verify lithicrivers.sig lithicrivers
```
(Requires importing our public GPG key first)