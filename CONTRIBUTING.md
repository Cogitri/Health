# Contributing to Health

Welcome to Health's first-time contributor docs. This document should help get you up to speed with contributing to Health.

## Making your first commit

### Useful documentation

Health uses Rust and gtk-rs. [The Rust Book](https://doc.rust-lang.org/stable/book/) and [gtk-rs tutorials](https://gtk-rs.org/docs-src/tutorial/) are excellent resources for getting started with them.
Health's documentation [is available here](https://cogitri.pages.gitlab.gnome.org/Health/libhealth/index.html) and should help you understand the basics of how Health is built. If you have any questions, feel free to reach out to the maintainers at the following Matrix channel: [#health:gnome.org](https://matrix.to/#/!kZVunSLsOSBXOdzKwz:gnome.org?via=gnome.org&via=cogitri.dev)!

### Working on the source

Now that you know how the internals of Health work you can start working on the bugfix or feature you want to implement. The easiest way to get started is using GNOME Builder. Simply open GNOME Builder, click on `Clone Repository...` and enter the repository URL (https://gitlab.gnome.org/Cogitri/Health). Afterwards click on `Clone Project` and you should be all setup. Pressing F5 or pressing the build button in the topbar should build Health for you.

### Committing your work

Health uses the Angular JS commit format guidelines since they make it very easy to see at a glance what a commit did. The long version is [here](https://gist.github.com/brianclements/841ea7bffdb01346392c), but the TL;DR is:

* Your commit should follow the style `<type>(<scope>): <subject>`, e.g. `fix(sync_list_box): allow re-authenticating`.
* `type` can be one of the following:
    * `chore` used for all things which aren't direct code improvements, e.g. updating dependencies, editing meson buildsystem files, changing CI files.
    * `feat` featurework (e.g. adding a new window).
    * `fix` anything that is only a bugfix and doesn't contain feature work.
    * `refactor` a code change that doesn't introduce new features or fixes bugs but e.g. makes the code easier to read.
    * `style` a commit that only does style fixes (e.g. running `cargo fmt`, but please try to only ever commit formatted code so this isn't necessary).
    * `test` adding tests or fixing existing tests.
* `scope` usually is the file you've worked in, the example mentioned previously touched the file `sync_list_box.rs`.
* `subject` is a short (under 80 characters) summary of what you changed. If you want to include more context, you can do that in the commit message's body, like so:

```
    fix(ui): remove can-focus & visible=true properties
    
    Visible=true isn't required anymore with GTK4 since widgets are visible by default.
    We (almost) always want can-focus to be true to allow selecting widgets via the keyboard.
    
    fixes #57
```

Note the `fixes #57`. This automatically closes the issue number `57` upon merging, if your commit fixes a certain issue it's good to include this.

If you're more comfortable using a GUI for your git work, you can read the guide at https://wiki.gnome.org/Newcomers/SubmitContribution.

### Submitting a merge request

Now that you've committed your work, it's time to submit it upstream (at the original repo), so it's included in Health. When you push new commits to your fork, Gitlab should print a link from which you can open a new merge request at Health. In case it doesn't, or you have lost the link, you can always manually create a merge request, see the [Gitlab docs](https://docs.gitlab.com/ee/user/project/merge_requests/creating_merge_requests.html) for that. Once you've submitted your MR, someone should review it shortly. Thanks for contributing to Health!
