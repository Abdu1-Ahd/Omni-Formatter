# Known Issues

This document lists the current problems in the project and our plans to fix them.

### Cloud Server Login Problem
**Problem:** Our automatic robots (CI/CD) get stuck logging into Cloudflare.
**Fix:** Provide a secret `CLOUDFLARE_API_TOKEN` to bypass manual login.

### Hardcoded Cloud Server Address
**Problem:** The cloud server address is hardcoded, preventing users from using private servers.
**Fix:** Move the address to a user-configurable settings file.

### Memory Leak
**Problem:** The fast engine occasionally forgets to clear memory over long periods.
**Fix:** Force the engine to reset itself after every 100 uses.

### Not Deleting Old Files
**Problem:** The extension accumulates old updates, wasting hard drive space.
**Fix:** Automatically delete older updates and retain only the two newest versions.

### HTML Formatting Mix-ups
**Problem:** Spaces can misalign when formatting HTML files that contain embedded JavaScript or CSS.
**Fix:** Implement an improved source-mapping system to track code blocks precisely.

### Slow Formatting on Massive Files
**Problem:** Formatting files larger than 20,000 lines takes too long.
**Fix:** Update the engine to format only the specific modified ranges instead of the entire file.

### Wrong Database Used for Downloads
**Problem:** The cloud server sometimes serves files from the slow database instead of fast file storage.
**Fix:** Force all downloads to route through Cloudflare R2 storage.

### Mixing Up "Enter" Key Styles
**Problem:** The formatter ignores OS-specific line endings (Windows vs. Mac).
**Fix:** Read user settings to enforce their preferred line ending style consistently.
