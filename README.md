
[![Build Status](https://travis-ci.org/cmatsuoka/oxdz.svg?branch=master)](https://travis-ci.org/cmatsuoka/oxdz)


<p>
<img alt="Oxidrizzle" src="https://github.com/cmatsuoka/oxdz/blob/master/logo.png" />
</p>

**Status: experimental, unstable, under development.**

This is an experimental multi-format mod player engine written in Rust, addressing the
main problems found during the development of libxmp: error handling, player accuracy,
maintainability.

Current mixing is using code chunks borrowed from libxmp to verify if the players are
correctly implemented. Sound quality will be improved later.


## Task list

* Standard M.K. loader :heavy_check_mark:
  * M.K. fingerprinting :heavy_check_mark:
* 15-instrument
  * UST/ST fingerprinting
* Multichannel
  * xCHN/xxCH fingerprinting
* Protracker mod player :heavy_check_mark:
  * Player variation for the same format (e.g. Noisetracker) :heavy_check_mark:
  * 15-instrument variant (Soundtrackers)
  * Amiga sound emulation
* Scream Tracker 2 support :heavy_check_mark:
* Scream Tracker 3 support :heavy_check_mark:
  * Play ST3 M.K. file (e.g. Sundance.mod)
  * Play ST3 xCHN/xxCH files
* Quirk table
* Fast Tracker II support
  * Play non-ST3 xCHN and xxCH files
* Better mixer
  * Better interpolation
  * Bidirectional loop
* Impulse Tracker support
* Module pre-scan
* Skip patterns forward/backwards
* Other language bindings
  * C
  * Something else (Go, Python, Java)
* Stable API
  * Public crate
* Player application
  * Proof of concept :heavy_check_mark:
  * CLI-based
  * Mobile app
  * Web-based
  * GUI player


## Nice to have, wishlist, etc

* Digitrakker player based on original sources
* Imago Orpheus player based on original sources
* SoundFX

