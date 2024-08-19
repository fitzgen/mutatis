/*!

# Comparisons to Other Libraries and Approaches

## What is the Goal?

Before we compare libraries and approaches, let's first establish our
intentions. It is this author's opinion that when it comes to testing code that
is safety-critical, handles untrusted inputs, or is otherwise foundational and
important, we should start with the following approach:

1. Factor unit tests out into functions that, given a test case, check that a
   property of the system is upheld. Or, said another way, these functions
   should act as an oracle, identifying whether bugs were triggered in the
   system when running a test case.

2. Write custom generators and/or mutators to produce pseudo-random test cases
   that we can use as inputs to those functions.

3. Combine those functions and pseudo-random test cases into short-running
   "property-based tests". These should be usable for quick feedback during
   development and catching low-hanging bugs in CI before pull requests
   merge. These should have known edge-cases enumerated and exercised,
   effectively acting as a mini-corpus.

4. Combine those functions and psedo-random test cases into long-running
   "fuzzers". These should not run in CI, but instead continuously in the
   background. They should leverage a corpus of known-interesting inputs and
   incorporate coverage-guided feedback. The goal is to find hidden bugs lurking
   in the depths.

I say we should start with the above approach because it really is just the
beginning. We should continuously strive to improve our testing strategies, and
we should also, whenever possible, do that with an eye towards generalization
away from testing single inputs and particular edge cases and towards something
we can use for both "property-based tests" and "fuzzing" simultaneously.

As we compare `mutatis` and its approach to other libraries and approaches,
we'll view that comparison through the above lens. If you disagree with the
above approach, that's your perogative and it's totally fine, but the following
comparisons probably won't be that useful to you. For example, because we aim to
do both "property-based testing" and "fuzzing" with the same components, we have
a natural bias towards modular libraries over frameworks with tightly entwined
test case generation/mutation and test runner, which can make it difficult to
use the test case generation/mutation independently from the test runner, or
plug in external test case generation/mutation into their test runner.

## How `mutatis` Helps Us Achieve the Goal

Note that part (1) of the goal approach is your own responsibility, being highly
domain-specific and requiring deep knowledge of the system under test. Helping
you with (2) is the primary reason why `mutatis` (and
[`arbitrary`](https://docs.rs/arbitrary)) exist. The
[`mutatis::check`][crate::check] module also provides a tiny framework for doing
(3) for you, but you can always use something else instead. And (4) should be
just [a small bit of glue code][crate::_guide::fuzzer_integration], once you've
selected a coverage-guided fuzzing engine, such as libfuzzer.

`mutatis` is intentionally decoupled from any particular property-based testing
framework or fuzzing engine to encourage reusing the same mutator and
property/oracle functions in both "property-based testing" and "fuzzing"
contexts.

## Comparison to Custom Generators

A generative approach to pseudo-random testing uses a *generator* to create a
pseudo-random test cases from scratch, feeds these test cases as input to the
system under test, and reports any test failures to the user:

```no_run
# enum TestResult { Ok, Err(()) }
# use TestResult::*;
# fn report_to_user<T>(_: &T, _: ()) {}
fn generative_pseudo_random_testing<T>(
    // A test-case generator. Provided by the user.
    generator: impl Fn() -> T,
    // A function to run the system under test with a generated test case,
    // returning a result that describes whether the run was successful or
    // not. Provided by the user.
    run_system_under_test: impl Fn(&T) -> TestResult,
) {
    loop {
        // Generate an input.
        let input = generator();

        // Run the input through the system under test.
        let result = run_system_under_test(&input);

        // If the system crashed, panicked, failed an assertion, violated an
        // invariant, or etc... then report that to the user.
        if let Err(failure) = result {
            report_to_user(&input, failure);
        }
    }
}
```

The idea is that the generator helps you efficiently explore the system under
test's state space by only generating valid inputs. You can better test a C
compiler by using [`csmith`](https://github.com/csmith-project/csmith) to
generate valid C programs, than by throwing random bytes at it. Probably those
random bytes are not syntactically valid C, let alone free from undefined
behavior! A large percent of your testing time would be spent attempting to
parse the random bytes and returning parser errors, rather than getting "deep"
into the compiler's pipeline and exercising its optimization passes and
code-generation backends.

On the other hand, with the mutation-based approach to pseudo-random testing
&mdash; the paradigm that `mutatis` subscribes to &mdash; we start with an
initial corpus of test cases and create new inputs by mutating existing corpus
members. We run each new input through the system under test, report test
failures the same as before, and if the new input was "interesting" (for
example, exercised new code paths in the system under test that weren't
previously covered in any other input's execution) then the new input is added
into the corpus for use in future test iterations:

```no_run
# struct Corpus<T> { _t: T }
# impl<T> Corpus<T> { fn choose_one(&self) -> &T { todo!() } }
# impl<T> Corpus<T> { fn insert(&mut self, _: T) { todo!() } }
# enum TestResult { Ok, Err(()) }
# impl TestResult { fn input_was_interesting(&self) -> bool { todo!() } }
# use TestResult::*;
# fn report_to_user<T>(_: &T, _: ()) {}
fn mutation_based_psuedo_random_testing<T>(
    // A corpus of test cases. Provided by the user.
    corpus: &mut Corpus<T>,
    // A function to pseudo-randomly mutate an existing input, creating a new
    // input. Provided by the user.
    mutate: impl Fn(&T) -> T,
    // A function to run the system under test with a generated test case,
    // returning a result that describes whether the run was successful or
    // not. Provided by the user.
    run_system_under_test: impl Fn(&T) -> TestResult,
) {
    loop {
        // Choose an old test case from the corpus.
        let old_input = corpus.choose_one();

        // Pseudo-randomly mutate that old test case, creating a new one.
        let input = mutate(old_input);

        // Run the input through the system under test.
        let result = run_system_under_test(&input);

        // If the system crashed, panicked, failed an assertion, violated an
        // invariant, or etc... then report that to the user.
        if let Err(failure) = result {
            report_to_user(&input, failure);
        }

        // If the input was interesting, for example if it executed previously-
        // unknown code paths, then add it into the corpus for use in future
        // test iterations.
        if result.input_was_interesting() {
            corpus.insert(input);
        }
    }
}
```

The idea here is that if a test case was determined to be interesting, then
mutated variants of it are also likely to be interesting, but might additionally
find some new interesting behavior as well. Swapping two variables in a valid C
program is more likely to produce another valid C program than generating random
bytes, and it is additionally more likely to find a bug lurking in the C
compiler's constant-propagation optimization pass.

So both generative and mutation-based pseudo-random testing are similar and are
similarly motivated.

An advantage of the mutation-based approach is that you can seed the corpus with
all of the test cases from your test suite, all your regression tests, Real
World(tm) inputs from the wild, etc... This lets you start testing interesting
inputs immediately, while a generator will have to get a lucky roll of the dice
to generate an interesting input. It is going to take a *very* long time for
`csmith` to just happen to generate the `sqlite.h` header file, but with a
mutation-based approach, you can just add that header to the initial corpus.

For a generator to create some test case, it has to have code to support
generating that test case. When new C++ language features come out, someone has
to add support to `csmith` for generating them, and that might lag behind when
you add support for that feature to your compiler. This is also true for custom
mutators: if the C++ program mutator doesn't have support for inserting
[SFINAE](https://en.wikipedia.org/wiki/Substitution_failure_is_not_an_error), it
will never create new test cases that contain SFINAE when the original input
didn't. However, fuzzing engines will often use your mutator as one strategy
among their many other builtin strategies that do things like insert or remove
arbitrary bytes, copy bytes from a corpus member into this one, etc... which
*can* create new uses of SFINAE despite neither the mutator nor the fuzzing
engine knowing anything about SFINAE.

Generators are typically implemented as decision trees, these trees are
imbalanced, and this leads to bias in which test cases are generated. The
following example will call `generate_a()` 50% of the time, `generate_b()` 25%
of the time, and `generate_c()` and `generate_d()` 12.5% of the time.

```no_run
# let random_choice = || true;
# let generate_a = || {};
# let generate_b = || {};
# let generate_c = || {};
# let generate_d = || {};
if random_choice() {
    generate_a();
} else if random_choice() {
    generate_b();
} else if random_choice() {
    generate_c();
} else {
    generate_d();
}
```

Although you wouldn't write this contrived example intentionally, it is
*extremely* easy to write it unintentionally, due to how the code is structured,
procedure boundaries, recursion, and because generators are often implemented as
inverse parsers. [John Regehr has a good blog post describing this problem in
more detail.](https://blog.regehr.org/archives/1700)

On the other hand, [Chen et al observed that mutation-based approaches can
escape this
predicament](https://github.com/wcventure/FuzzingPaper/blob/master/Paper/PLDI16_JVM.pdf):
we can use [Markov Chain Monte
Carlo](https://en.wikipedia.org/wiki/Markov_chain_Monte_Carlo) to uniformly
sample, given sufficient time, from the distribution of all test cases that
could possibly be created by applying any of our mutators any number of
times. Furthermore, even when choosing a single mutation to apply to a
particular value, `mutatis` avoids decision trees and their biases. Instead it
enumerates the mutations that could be applied to this value, and then selects
one uniformly.

It is also worth mentioning that we can create a generator from a corpus and
mutator; we can't do the reverse.

```no_run
# struct Corpus<T> { _t: T }
# impl<T> Corpus<T> { fn take_one(&mut self) -> T { todo!() } }
# impl<T> Corpus<T> { fn insert(&mut self, _: T) { todo!() } }
# enum TestResult { Ok, Err(()) }
# fn generative_pseudo_random_testing<T>(
#     _generator: impl FnMut() -> T,
#     _run_system_under_test: impl Fn(&T) -> TestResult,
# ) { }
fn generation_via_mutation<T: Clone>(
    // A corpus of test cases. Provided by the user.
    corpus: &mut Corpus<T>,
    // A function to pseudo-randomly mutate an existing input, creating a new
    // input. Provided by the user.
    mutate: impl Fn(&T) -> T,
    // A function to run the system under test with a generated test case,
    // returning a result that describes whether the run was successful or
    // not. Provided by the user.
    run_system_under_test: impl Fn(&T) -> TestResult,
) {
    // Create a generator from a corpus and a mutator.
    let mut generator = || {
        // Take an old test case from the corpus.
        let old_input = corpus.take_one();

        // Mutate it into a new test case.
        let new_input = mutate(&old_input);

        // Insert the new test case into the corpus, to replace the old one.
        // Alternatively, we could leave the old test case and use a side
        // channel to update the corpus based on whether the new input was
        // "interesting" or not.
        corpus.insert(new_input.clone());

        // We "generated" this new input!
        new_input
    };

    // Do "generative" testing via mutation.
    generative_pseudo_random_testing(generator, run_system_under_test)
}
```

Finally, there is nothing stopping you from pursuing both generative and
mutation-based approaches, if you have the time and can put in the effort. If I
was responsible for a JavaScript engine that handled untrusted inputs from the
Web, I would never fuzz it with *only* my custom JS mutator, or *only* my custom
JS generator. I would also use other folks' generators even if they were
"redundant" and I would also throw raw bytes at the JS engine, not just
structured inputs coming out of generators and mutators. I would use absolutely
everything I could get my hands on.

## Comparison to Fuzzing

`mutatis` is not an alternative to fuzzing, it is a tool to enhance
pseudo-random testing like fuzzing.

You use `mutatis` to write a custom mutator that creates new pseudo-random
inputs by mutating old, existing inputs. That mutator can maintain your input
type's invariants, structure, and validity. For example, you can avoid
generating syntactically invalid inputs that bounce off your parser, checksums
that fail to match their associated data, etc... By using this custom mutator,
the fuzzer better and more efficiently explores the system under test's state
space.

## Comparison to Property-Based Testing

Property-based testing (PBT) and fuzzing are both just pseudo-random testing at
the end of the day, they just happened to evolve out of different
communities. As such, everything in the "Comparison to Fuzzing" section above
broadly applies here as well: `mutatis` is complimentary to and an enhancement
for PBT, not a competing alternative. That said, PBT and fuzzing do have
different historical tendencies, and the strengths of one are often the
weaknesses of the other.

Fuzzing has often focused on finding exploitable security bugs, and often runs
continuously over a period of months. However, fuzzers don't typically know too
much about the system under test's input's structure, or even when they do, it
is assumed to have some serialized or textual at-rest representation. This means
that testing with input data structures that require a bunch of fiddly
invariants to be maintained (checksums, valid compressed data, correct lengths
for run-length encodings, etc...) is a little harder, while maintaining a corpus
of fuzzing inputs to start the state space exploration from is a little easier.

PBT, on the other hand, has focused more on creating psuedo-random structured
inputs to functions that test that some property is upheld. PBT doesn't require
a serialization or stringification step to turn the structured input into the
at-rest representation that the system under test consumes, because the system
under test is just a function that can simply be invoked, passing the input as
an argument. That is simple and efficient, but makes maintaining a corpus of
interesting inputs we've previously discovered a little trickier. PBT is also
often used as "unit testing++" and comes with an expectation that you can run
property-based tests in a similar amount of time as running unit tests. That
leaves less patience for months-long, continuous testing.

As described in the goals section at the beginning of this document, we should
be doing both short-running pseudo-random testing in local development and
gating on those tests in CI, as well as doing coverage-guided, pseudo-random
testing in the background, continuously. Both short- and long-running variants
should leverage knowledge of the input structure (i.e use custom mutators and/or
generators). Both short- and long-running variations should leverage corpora of
test cases from old regressions and other edge cases.

## Comparison to Formal Methods and Verification

In addition to the testing approach laid out at the top of this document, we
should pursue formal methods to verify various properties of our systems and
prove their correctness, but this topic is mostly outside the scope of this
document.

It is, however, worth noting that translation validators are fantastic
property/oracle functions for pairing with generators and/or mutators in a
larger pseudo-random testing context. For an example of using translation
validation in a fuzzing context, [check
out](https://cfallin.org/blog/2021/03/15/cranelift-isel-3/) how the `regalloc2`
project generates random virtual-register programs, runs register allocation on
them to create hardware-register programs, and then runs a symbolic checker that
proves that the resulting program is a valid translation of the input program.

Finally, even if a system is formally proven correct, it is still possible for
bugs to exist in the implementation. For example, the formal proof might be for
a subtly different property than the one you actually care about, or it might
make some simplifying assumptions, like the absence of out-of-memory
errors. Pseudo-random testing, which `mutatis` helps with, provides the best
additional assurance.

## Comparison to `arbitrary`

[`arbitrary`][arbitrary] is very similar to `mutatis`, but takes a generative
approach, rather than a mutation-based approach. `arbitrary` is a library for
generating random values of a type, while `mutatis` is a library for mutating
existing values of a type. Everything in the "Comparison to Custom Generators"
section above applies to this comparison, including the bit about the approaches
being complimentary.

Both libraries are designed to be modular and make very few assumptions about
what larger testing and fuzzing frameworks you integrate them with. Both are
maintained or mostly-maintained by the same person: this author. Looking at our
goal approach for testing, `arbitrary`'s main purpose is also to help you do
(2), just like `mutatis`. The [`arbtest`][arbtest] crate provides a mini
property-based testing framework on top of `arbitrary` which can help you do
(3), similar to the [`mutatis::check`][crate::check] module. And, once again,
similar to `mutatis`, (4) is just a little bit of glue code to integrate with
`libfuzzer`. The two crates live in the same niche and have the same high-level
goals, they just take slightly different approaches to achieve them.

It is worth pointing out that `arbitrary` can *sort of* take advantage of the
fuzzer's underlying mutation-based nature because it treats the fuzzer's raw
bytes as a "DNA string" that it uses to generate a new test case, rather than
using a simple random number generator to make all decisions. In theory, small
changes to the DNA string should result in small changes to the generated test
case. However, this is not a silver bullet: any change to an `Arbitrary`
implementation invalidates the corpus, so that it doesn't reflect the set of
generated values it originally did. This is frustrating when all your OSS-Fuzz
bugs get opened to the public because they "got fixed" and the crashes don't
reproduce anymore. It also means that the only way to create a corpus is to
start generating stuff from scratch. You can't seed the corpus with programs
that previously triggered misoptimizations in your compiler or invalid
almost-URLs that your URL parser accidentally accepted as valid in the
past. Finally, `arbitrary` implementations definitely, 110% suffer from the
imbalanced, biased decision tree problem described in the "Comparison to Custom
Generators" section. `mutatis` has none of these asterisks.

So which crate is better? Who knows. `arbitrary` has been around longer and at
the time of writing is certainly used more in anger. Play with each of them and
see which you vibe with, which feels easier to build with and maintain, and
which results in higher fuzzing throughput and deeper coverage. Or use both, if
you can spare the effort.

[arbitrary]: https://docs.rs/arbitrary
[arbtest]: https://docs.rs/arbtest

## Comparison to `proptest`

[`proptest`](https://proptest-rs.github.io/proptest/intro.html) is a popular
property-based testing framework for Rust. It has a large user base and its
tires have therefore been thoroughly kicked. It handles both test case
generation (potentially implemented via mutation under the covers) and running
your tests.

`proptest` has many, high-quality strategies and combinators for test case
generation. Certainly more and higher-quality strategies than those that exist
in `mutatis` at the time of writing; I hope to take inspiration from this aspect
of `proptest` as `mutatis` matures.

`proptest` is not designed for reusing your generation strategies and properties
with fuzzing engines, but I think doing so is likely possible if you put in the
elbow grease and maintain discipline in the way that you write and factor your
tests. But it doesn't naturally push you in that direction. On the other hand,
`mutatis` is intentionally decoupled from any particular property-based testing
framework or fuzzing engine to encourage this style of reuse.

You can maintain a corpus of test cases that triggered historical failures with
`proptest` via its ["failure
persistence"](https://proptest-rs.github.io/proptest/proptest/failure-persistence.html). However,
that corpus consists of RNG seeds that ultimately generated the failing test
case, and you cannot initialize the corpus with arbitrary test cases you happen
to have on hand (like `sqlite.h` for testing your C compiler). It also means
that if you change the implementation of your generator, then the corpus doesn't
reflect the set of values that it previously did, just like with `arbitrary`. On
the other hand, the [`mutatis::check`][crate::check] mini property-based testing
framework allows you to supply any set of values you want, directly, as the
initial corpus. This means that you don't need separate unit tests for your
known edge cases, you just throw them in the initial corpus.

I don't think there there is anything fundamentally stopping you from using
`mutatis`-based mutators to implement `proptest::Strategy`, or vice versa,
should you wish to combine the two crates for whatever reason.

## Comparison to `quickcheck`

[`quickcheck`](https://docs.rs/quickcheck/latest/quickcheck/) is a
property-based testing framework for Rust. It is very popular and I think it may
even be the first property-based testing framework that existed for Rust. It has
a large user base and its tires have therefore been thoroughly kicked.

`quickcheck` takes a generative approach to pseudo-random testing, while
`mutatis` takes a mutation-based approach. See the "Comparison to Custom
Generators" section above; all of that applies to quickcheck as well.

`quickcheck` is simple and easy to use. However, you cannot implement
`quickcheck::Arbitrary` multiple times for a single type, which means that you
are limited to one generation strategy and one shrinking strategy for each type,
and if you don't like the way it generates arbitrary `char`s you are out of
luck. `mutatis` is less simple, and takes a little more effort to understand the
core traits and get things up and running, but it provides greater flexibility
with no limit on the number of mutation strategies for a given type.

`quickcheck` does not support maintaining a corpus. You cannot ensure that test
cases that triggered historical failures are checked going forward. You cannot
supply initial edge cases that you always want to check. You need to write
separate unit tests for these things. On the other hand, the
[`mutatis::check`][crate::check] mini property-based testing framework allows
you to supply any set of values you want, directly, as the initial corpus
including test cases that triggered historical failures and any other edge case
you can think of.

`quickcheck` is not designed for reusing your generation strategies and
properties with fuzzing engines, but doing so is possible if you put in the
elbow grease and maintain discipline in the way that you write and factor your
tests. But it doesn't naturally push you in that direction. On the other hand,
`mutatis` is intentionally decoupled from any particular property-based testing
framework or fuzzing engine to encourage this style of reuse.

*/
