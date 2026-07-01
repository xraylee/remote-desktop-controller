# Root Directory Cleanup Plan

**Date**: 2026-06-29  
**Target**: Organize 23 .md files and 22 .sh files from root directory

---

## File Classification

### Keep in Root (6 essential files)
✅ `README.md` - Project homepage
✅ `CHANGELOG.md` - Version history
✅ `CODE_OF_CONDUCT.md` - Community standards
✅ `CONTRIBUTING.md` - Contribution guide
✅ `SETUP.md` - Setup instructions
✅ `TODO.md` - Active task list

---

### Move to docs/reports/ (13 status/test reports)
- `COMPILE_VERIFICATION.md`
- `COMPLETION_SUMMARY.md`
- `FIX_SUMMARY.md`
- `PHASE4.1_STATUS.md`
- `PROJECT_STATUS_2026-06-29.md`
- `SIGNALING_CONNECTION_DIAGNOSIS.md`
- `STRUCTURE_REORGANIZATION_REPORT.md`
- `TEST_EXECUTION_SUMMARY.md`
- `VERIFICATION_CHECKLIST.md`
- `WEB_ADMIN_TEST_COMPLETE.md`
- `WEB_ADMIN_UI_LOCALIZATION.md`
- `WEB_SERVICE_TEST_COMPLETE.md`
- `WEB_SERVICE_TEST_SUMMARY.md`

---

### Move to docs/guides/ (4 quick reference docs)
- `PROJECT_STRUCTURE.md`
- `QUICK_REFERENCE.md`
- `QUICK_TEST_GUIDE.md`

---

### Move to scripts/build/ (7 build scripts)
- `build_and_run.sh`
- `build_ffi.sh`
- `build_ice_tools.sh`
- `check_build.sh`
- `quick-fix.sh`
- `quick_start.sh`
- `test_target.sh`

---

### Move to scripts/deployment/ (4 deploy scripts)
- `deploy_backend.sh`
- `deploy_minimal.sh`
- `logs_backend.sh`
- `stop_backend.sh`

---

### Move to scripts/testing/ (6 test scripts)
- `run_real_screen_capture_test.sh`
- `test_api.sh`
- `test_controller.sh`
- `test_hardware_encoder.sh`
- `TEST_COMMANDS.sh`
- `verify-test-docs.sh`

---

### Move to scripts/setup/ (3 setup scripts)
- `setup_environment.sh`
- `setup_xcode.sh`

---

### Move to scripts/git/ (3 git helper scripts)
- `git_commit.sh`
- `git_commit_phase4.1.sh`

---

### Move to scripts/diagnostics/ (1 diagnostic script)
- `diagnose_auth.sh`

---

## Execution Steps

1. Create target directories
2. Move files
3. Update scripts/README.md
4. Update docs/README.md
5. Create redirect notes in root (optional)

---

## Result

**Before**: 28 files in root (6 essential + 22 other)  
**After**: 6 files in root (essential only)  
**Cleanup**: 22 files organized into proper directories
