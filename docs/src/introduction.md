# Introduction

Welcome to `rhai_trace`, a simple library for better Rhai errors.

Rhai has bad errors because it is designed to be a general purpose library and most of the time, good errors are not needed. But in some certain cases, some poeple may require advanced errors and that is where `rhai_trace` comes in.

It provides a `BetterError` structure which contains important fields like `start` and `end` which is very important for diagnostic like errors.
