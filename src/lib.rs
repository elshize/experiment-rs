// MIT License
//
// Copyright (c) 2019 Michał Siedlaczek
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#[macro_use]
pub mod process;

/// Indicator of whether the output should be verbose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Verbosity {
    Brief(usize),
    Verbose,
}

/// Returns [`Verbosity`](Verbosity.t.html) based on a condition.
/// ```
/// # use experiment::{verbose_if, Verbosity};
/// assert_eq!(verbose_if(true, 10), Verbosity::Verbose);
/// assert_eq!(verbose_if(false, 10), Verbosity::Brief(10));
/// ```
pub fn verbose_if(verbose: bool, max_args: usize) -> Verbosity {
    if verbose {
        Verbosity::Verbose
    } else {
        Verbosity::Brief(max_args)
    }
}
