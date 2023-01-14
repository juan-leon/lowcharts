0.5.8
=====

Features:

* Support logarithmic scale in histograms via `--log-scale` flag.

0.5.7
=====

* Add debian packages to releases

* Add ARM build in releases

0.5.6
=====

* Add LICENSE file to distribution in binary package

0.5.4
=====

* Add AUR installation info

0.5.2
=====

Doc improvements

0.5.0
=====

Features:

* Allow to be use the `lowcharts` as a library.  See README for an example.  API
  documentation: https://docs.rs/lowcharts/latest/lowcharts/

0.4.4
=====

Features:

* Use "human units" by default in `hist` and `plot` sub-commands.  Big numbers
  (those with many digits) can be hard to read.  `lowcharts` will use some
  heuristics to use units when helpful (for instance, "412.7 M" as opposed to
  "412723763251.327").  Heuristics will take into account the range to be
  displayed and will keep units consistent in all of the visualization.  Command
  line option `--precision` is available to deactivate this feature and request
  for an arbitrary number of decimals.

Bug fixes:

* In time histograms, do not panic if all input timestamps are the same.

0.4.3
=====

Bug fixes:

* Do not truncate numbers in Stats section of `hist` and `plot` sub-commands if
  they have many digits.

0.4.2
=====

Features:

* Implement the sub-command `common-terms`.  Use it to display an histogram with
  the number of occurrences of the most common terms of the input.  Use it with
  `--regex` if the input is not filtered.  Example:

```
# Figure out the most used syscalls when listing a directory
#
$ strace ls -l 2>&1 | lowcharts common-terms --lines 5 --regex '(.*?)\('
Each ∎ represents a count of 1
[  openat] [40] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
[    mmap] [36] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
[getxattr] [29] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
[   close] [29] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
[   fstat] [25] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
```

0.4.1
=====

Bug fixes:

* Allow negative values in min and max arguments

0.4.0
=====

Features:

* Implement the sub-command `split-timehist`.  It mixes up the time histogram
  and bar chart in a single visualization.  See README for an usage example.

0.3.0
=====

Features:

* Implement the sub-command `timehist`.  It displays the frequency of logs that
  match a regex (by default any log that is read by the tool).  The sub-command
  can autodetect the most common (in my personal and biased experience)
  datetime/timestamp formats: rfc 3339, rfc 2822, python `%(asctime)s`, golang
  default log format, nginx, rabbitmq, strace -t (or -tt, or -ttt), ltrace,...
