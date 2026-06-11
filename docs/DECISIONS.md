# Why We Made Certain Choices

This document explains the reasons behind the big decisions we made while building OmniFormatter.

## 1. Using Rust
**Why:** We wanted the formatter to run exactly the same way everywhere (in VS Code, on the terminal, and on the internet). 
**Result:** Rust makes the tool very fast and guarantees it works identically everywhere without needing extra software installed.

## 2. Copying Prettier Exactly
**Why:** People don't like changing how their code looks.
**Result:** For web languages, we format the code exactly the same way Prettier does. This makes switching to OmniFormatter easy.

## 3. Using Tree-Sitter
**Why:** Simple text searches (regex) are not smart enough to understand complex code.
**Result:** Tree-sitter reads code like a human reads a sentence, understanding the grammar. It is much more reliable.

## 4. Hosting on Cloudflare
**Why:** We need to send plugins to users very fast, no matter where they are in the world.
**Result:** Cloudflare servers are everywhere, so downloads are extremely fast and cheap.

## 5. Digital Signatures
**Why:** We want to make sure nobody uploads a virus pretending to be a formatter.
**Result:** Every plugin is locked with a digital key. If the key doesn't match, we block the download.

## 6. Background Workers
**Why:** Formatting should never freeze the editor while you are typing.
**Result:** We send the work to an invisible background helper. Your editor stays fast.

## 7. Cloudflare D1 Database
**Why:** We needed a fast way to keep track of who uploaded which plugin.
**Result:** This database is fast and easy to search.

## 8. Sandboxed Command Line
**Why:** We need a way to run the formatter automatically on servers.
**Result:** We built a tool that runs the plugins in a locked box, ensuring complete safety.

## 9. Special Memory Management
**Why:** The standard way of managing computer memory caused crashes.
**Result:** We switched to a faster, safer memory manager (Talc) which fixed all crashes.

## 10. Doing It Twice Check
**Why:** A formatter should never change the code if it is already perfect.
**Result:** Our tests constantly check that formatting the code a second time does absolutely nothing.
