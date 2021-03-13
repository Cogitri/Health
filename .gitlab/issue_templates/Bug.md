<!--
    Text within the <!- -> won't be visible in the final bug report.

    Thanks for your bug report! First off, please provide a short summary of what went wrong:
-->

## Summary

When I did X Health crashed.

Steps to reproduce:

1. Open Health
2. Do X
3. Crash occurs

## Environment

**Health Version**: <!-- The version of Health you were using when the bug occurred. Check the "About Health" dialog for this information -->

**OS Version**: <!-- Operating system version, e.g. Fedora 31 -->

**Installation Source**: <!-- Where you installed Health from, e.g. Flathub, AUR, or distro repositories -->

<!--
    If you experienced a crash Health, you can put the backtrace (which details where things went wrong) put it here.
    If you use the flatpak version of Health from Flathub, you can generate it like so:

    ```sh
        flatpak install dev.Cogitri.Health.Debug
        flatpak-coredumpctl dev.Cogitri.Health
    ```

    You should be dropped into gdb's shell now. Simply type `bt` now and paste the result below.
-->

## Backtrace

```
#0  0x00007f802feb395a in some_function
#1  0x00007f802f939b26 in  ()
#5  0x00007f802feab9c3 in some_other_function
[...]
```
