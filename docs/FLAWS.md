# Known Problems and Bugs

This document lists the current problems in the project and how we plan to fix them.

### F-001: Cloud Server Login Problem
**Problem:** Our cloud server requires a human to click and log in on a browser. Our automatic robots (CI/CD) can't do this, so they get stuck.
**How to Fix:** Give the robots a secret password (`CLOUDFLARE_API_TOKEN`) so they can log in without clicking.

### F-002: Hardcoded Cloud Server Address
**Problem:** The address to our cloud server is written directly into the code. This makes it hard for other people to use their own private servers.
**How to Fix:** Move the address to a settings file that users can change easily.

### F-003: Memory Leak (Forgetting to Clean Up)
**Problem:** Sometimes, our fast engine forgets to throw away old data from its memory. If it runs for many days, it might use up all the computer's memory.
**How to Fix:** Tell the engine to restart itself completely after every 100 uses, so its memory is fresh again.

### F-004: Not Deleting Old Files
**Problem:** When the extension downloads new updates, it never deletes the old versions. Over time, this takes up too much hard drive space.
**How to Fix:** Write a rule to automatically delete all old updates and only keep the two newest ones.

### F-005: HTML Formatting Mix-ups
**Problem:** When formatting HTML files with JavaScript or CSS inside them, the line numbers can get confused, causing spaces to look slightly wrong.
**How to Fix:** Make a better map to track exactly where each piece of code belongs.

### F-006: Slow Formatting on Very Large Files
**Problem:** If a file has over 20,000 lines, formatting it while typing can take too long because it reads the whole file every time.
**How to Fix:** Teach the engine to only read the exact small part of the file that was just changed.

### F-007: Wrong Database Used for Downloads
**Problem:** Sometimes the cloud server sends files using a slow database instead of the fast file storage. This will get too slow if many people use it.
**How to Fix:** Force the server to always use the fast file storage (R2) for downloads.

### F-008: Mixing Up "Enter" Key Styles
**Problem:** Windows computers and Mac computers save the "Enter" key (new lines) differently. The formatter currently ignores this and might mix them up.
**How to Fix:** Read the user's settings to see which style of new line they want, and use exactly that.
