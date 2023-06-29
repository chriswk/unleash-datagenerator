# Command-Line Help for `unleash-datagenerator`

This document contains the help content for the `unleash-datagenerator` command-line program.

**Command Overview:**

* [`unleash-datagenerator`↴](#unleash-datagenerator)

## `unleash-datagenerator`

**Usage:** `unleash-datagenerator [OPTIONS]`

###### **Options:**

* `-c`, `--count <COUNT>` — How many feature toggles should be generated

  Default value: `1000`
* `-s`, `--strategies-per-feature <STRATEGIES_PER_FEATURE>` — How many strategies per feature toggle. This does mean that total datapoints will equal count * this

  Default value: `10`
* `-e`, `--environment <ENVIRONMENT>` — Which environment should we create the strategies under. This environment needs to already exist

  Default value: `development`
* `--print-to-shell` — Don't POST feature toggles and strategies. Only output the json to stdout

  Default value: `false`
* `-u`, `--unleash-url <UNLEASH_URL>` — Where is the Unleash instance you'd like to generate data for

  Default value: `http://localhost:4242`
* `-p`, `--project-name <PROJECT_NAME>` — Name of the project to generate features under. If you're using Unleash OSS, leave the default, it's the only project that exists

  Default value: `default`
* `-a`, `--api-key <API_KEY>` — Needs to be an admin API key or a service account token with access to CREATE_FEATURE, CREATE_FEATURE_STRATEGY and UPDATE_FEATURE_ENVIRONMENT permissions
* `-m`, `--markdown` — Generate clap help file in markdown format and exit

  Default value: `false`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

