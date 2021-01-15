# Codestyle

Most of the codestyle is enforced via [vala-lint](https://github.com/vala-lang/vala-lint). You can run it in the root of the repository via `ninja -C build lint` or directly via `io.elementary.vala-lint --config .vala-lint.conf src/`. It enforces:

* Indention with spaces
* No trailing whitespace
* Spaces after function calls (line so: `func (arg1, arg2);`)
* Not using `snake_case` for naming variables & functions

Additionally, the following things should be kept in mind:

## Directory Structure

The project is divided into multiple directories:

* Core - Core classes like Application are in here. These don't fit into other categories.
* Model - Model classes (without any UI code) live here, like the different `GraphModel`s that hold the data for drawing graphs.
* Sync - Classes used for synching with 3rd party providers are stored here.
* Views - Toplevel views live here.
* Widgets - Widgets which can be integrated into other views live here.
* Windows - Windows and Dialogues live here.

## Class Structure

Classes should roughly be structured like this:

```
public class Foo {
    [GtkChild]
    private Gtk.Widget gtk_child;

    private Object private_member;
    private Object _property_member_custom;
    protected Object protected_member;
    public Object public_member; // You may want to use properties instead of making a public member though

    public Object property_member { get; set; }
    public Object property_member_custom {
        get {
            return this._private_property;
        }
        set {
            this._private_property = value;
        }
    }

    public signal void sig ();

    static construct {}

    construct {}

    public Foo () {}

    ~Foo() {}

    public public_func () {}

    protected protected_func () {}

    private private_func () {}

    [GtkHandler]
    private private_handler_func () {}
}
```

## One Public Class per File

This makes it easy to find classes in the source files Health has.

## Explictly use namespaces other than GLib

Please don't `use` other namespaces and instead prefix variable types (e.g. `Gtk.Widget`).

## Use `this`

Please use `this.` to access member variables to avoid accidentally accessing other variables and to make clear what you're accessing a member variable.
