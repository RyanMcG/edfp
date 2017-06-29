# edfp

**ed**it **f**ile **p**aths

---

#### WIP

Currently this program *does* behave as described below, but maybe not as you would expect in all cases.

## Usage

```bash
ls | edfp
```

*edfp* reads stdin, assumes each line is a path to a file object (file or
directory) and allows for easy manipulation of those objects.

1. Changing the name of the file will rename it.
2. Clearing the line will delete it.

### Example

```bash
$ mkdir playground && cd playground
$ touch a b c
$ ls
a b c
$ ls | edfp
```
Your `VISUAL` (or `EDITOR` if unset) program of choice will be opened with each line representing a file path:

```
a
b
c
```

Let's say you edit it to this:

```
x

y
```

Write and quit your editor. You'll be prompted like so:

```
DELETE the following files:

        b

RENAME the following files:

        a → x
        c → y

Would you like to make the changes described above (y/n)?
```

Type `y` and hit enter!

```bash
$ ls
x y
```

You renamed *a* → *x* and *c* → *y* and deleted *b*. Good job!


## Build

```bash
cargo build
```

## License

MIT. See [LICENSE][].

[LICENSE]: https://raw.githubusercontent.com/RyanMcG/edfp/master/LICENSE
