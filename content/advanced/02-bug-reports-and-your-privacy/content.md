# Bug reports and your privacy

This is exactly what the bug reporter collects, what it removes, and what it never touches. It is the same statement shown in the reporter and in settings.

## What a report contains

- Recent log lines from the app and warnings from the bread daemon.
- The crash location: which program, which function offsets, the build it came from. Never memory contents.
- Names of recent bread events. Event details are dropped unless they are known to be harmless (battery percentage, whether you are online, a USB device's vendor and product id).
- Basic system facts: OS version, kernel, graphics driver, compositor version, bread app versions.
- Anything you type in the notes box.

## What is replaced before anything is stored or shown

- Your name, home folder and computer name.
- IP addresses, MAC addresses, email addresses, device serial numbers, Wi-Fi network names, window titles.
- Passwords, access tokens, keys, and the query strings and logins in web addresses.
- Long random-looking strings that look like secrets.

This happens twice: when the daemon first stores a log line, and again when the report is built. You read the result before deciding. Text matching is best effort, which is why you always get to read it first.

## What is never collected

- Screenshots. Process memory. Your files. Your clipboard contents. Anything from apps you have excluded.

## Where it goes and how long it stays

- Nothing leaves your computer unless you choose Send. Sending is off by default.
- A sent report goes to the bread project's own server over HTTPS. The server does not store your IP address, and checks the report again for personal values.
- Raw reports are deleted after 90 days. Only anonymous counts are kept so a bug can be tracked.
- You can delete a report you sent at any time with `breadbug delete`, using the receipt number you were given.
- Reports from one install are linked only by a random number that changes every 30 days. There is no account and no device id.

## Your controls

- Turn sending off or on in bos-settings (Bug reports) or with `breadbug config upload`.
- Exclude an app so nothing is stored for it at all.
- Delete everything stored locally from the same settings page.
- Save a report to a file instead of sending it.
