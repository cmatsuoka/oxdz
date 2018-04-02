
[![Build Status](https://travis-ci.org/cmatsuoka/oxdz.svg?branch=master)](https://travis-ci.org/cmatsuoka/oxdz)


<p>
<img alt="Oxidrizzle" src="https://github.com/cmatsuoka/oxdz/blob/master/logo.png" />
</p>

**Status: experimental, unstable, under development.**

This is an experimental multi-format mod player engine written in Rust, addressing the
main problems found during the development of libxmp: error handling, player accuracy,
maintainability.

Current mixing is using code chunks borrowed from libxmp to verify if the players are
correctly implemented. Sound quality will be improved over time.

[Supported module formats](https://github.com/cmatsuoka/oxdz/wiki/Supported-formats)

[Experimental CLI player](https://github.com/cmatsuoka/0xd2)


## Task list

### Must have

* Formats and players
  * Standard M.K. loader               :heavy_check_mark:
  * M.K. fingerprinting                :heavy_check_mark:
  * Multichannel module loader         :heavy_check_mark:
  * xCHN/xxCH fingerprinting           :heavy_check_mark:
  * Protracker M.K. mod player         :heavy_check_mark:
  * Noisetracker player                :heavy_check_mark:
  * 15-instrument (Soundtracker)       :heavy_check_mark:
  * Ultimate Soundtracker              :heavy_check_mark:
  * Scream Tracker 2                   :heavy_check_mark:
  * Scream Tracker 3                   :heavy_check_mark:
    * Play ST3 M.K. file (e.g. Sundance.mod)  :heavy_check_mark:
    * Play ST3 xCHN/xxCH files
  * Fast Tracker II support
    * Also play non-ST3 xCHN and xxCH files
  * Impulse Tracker support
* Quirk table
* Mixer
  * Better interpolation               :heavy_check_mark:
  * Amiga sound emulation              :heavy_check_mark:
  * Bidirectional loop
* Module pre-scan                      :heavy_check_mark:
* Skip patterns forward/backwards      :heavy_check_mark:
* Other language bindings
  * C
* Stable API
  * Documentation and examples
  * Public crate
* Player application
  * CLI-based                          :heavy_check_mark:


### Nice to have, wishlist, etc

* Format support
  * Digitrakker player based on original sources
  * Imago Orpheus player based on original sources
  * Multitracker
  * SoundFX
* Other language bindings
  * Something else (Go, Python, Java, etc)
* Player application
  * Mobile app
  * Web-based
  * GUI player

