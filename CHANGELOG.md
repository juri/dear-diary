# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Support for more date formats: no-spaces 2020-09-21T12:34+1000, local timezone 2020-09-21 13:37, 2020-09-21T13:37, date-only 2020-09-21, time-only 13:37, 1337, 1:23am, 02:45PM
- Edit entries with `diary edit`
- Index tags (spelled #word or #(many words)#), allow search with them
- Take diary path and name as a parameter, construct full path from them
- Add new entries when running without other parameters
- List entries with `ddiary list`, show a single entry with `ddiary show`
- Force stdin input with `ddiary add --stdin`
- Allow specification of the date of the new entry with `ddiary add --date`
- Fix date input to always allow arbitrary time zones
