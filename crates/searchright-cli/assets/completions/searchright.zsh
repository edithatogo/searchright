#compdef searchright

autoload -U is-at-least

_searchright() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
":: :_searchright_commands" \
"*::: :->searchright" \
&& ret=0
    case $state in
    (searchright)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-command-$line[1]:"
        case $line[1] in
            (init)
_arguments "${_arguments_options[@]}" : \
'--target=[]:TARGET:_files' \
'--apply[Apply the write. Without this flag the command is a dry run]' \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(plan)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
":: :_searchright__subcmd__plan_commands" \
"*::: :->plan" \
&& ret=0

    case $state in
    (plan)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-plan-command-$line[1]:"
        case $line[1] in
            (validate)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-amendment)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(evaluate-governance)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':policy:_files' \
':request:_files' \
&& ret=0
;;
(workflow)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__plan__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-plan-help-command-$line[1]:"
        case $line[1] in
            (validate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-amendment)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(evaluate-governance)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(workflow)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(source)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
":: :_searchright__subcmd__source_commands" \
"*::: :->source" \
&& ret=0

    case $state in
    (source)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-source-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(authorise-endpoint)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
':endpoint:_default' \
&& ret=0
;;
(validate-discovery-run)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(discovery-candidates)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(verify-provider-component)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':manifest:_files' \
':component:_files' \
&& ret=0
;;
(plan-licensed-request)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':profile:_files' \
':strategy:_files' \
':endpoint:_default' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__source__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-source-help-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(authorise-endpoint)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-discovery-run)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(discovery-candidates)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(verify-provider-component)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(plan-licensed-request)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(strategy)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
":: :_searchright__subcmd__strategy_commands" \
"*::: :->strategy" \
&& ret=0

    case $state in
    (strategy)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-strategy-command-$line[1]:"
        case $line[1] in
            (validate)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(compile)
_arguments "${_arguments_options[@]}" : \
'--dialect=[]:DIALECT:(pubmed ovid-medline embase europe-pmc cinahl-ebsco psycinfo-ovid scopus web-of-science crossref openalex clinicaltrials-gov generic-boolean)' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-search)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-standard-pack)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-standard-assessment)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-benchmark-report)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__strategy__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-strategy-help-command-$line[1]:"
        case $line[1] in
            (validate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(compile)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-search)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-standard-pack)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-standard-assessment)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-benchmark-report)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(run)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
":: :_searchright__subcmd__run_commands" \
"*::: :->run" \
&& ret=0

    case $state in
    (run)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-run-command-$line[1]:"
        case $line[1] in
            (authorise-endpoint)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
':endpoint:_default' \
&& ret=0
;;
(verify-audit)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(verify-workflow-trace)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(inspect-content)
_arguments "${_arguments_options[@]}" : \
'--subject-id=[]:SUBJECT_ID:_default' \
'--policy=[]:POLICY:(data-only sanitise-then-data-only human-inspection-required)' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-document-evidence)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__run__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-run-help-command-$line[1]:"
        case $line[1] in
            (authorise-endpoint)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(verify-audit)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(verify-workflow-trace)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(inspect-content)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-document-evidence)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(import)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
":: :_searchright__subcmd__import_commands" \
"*::: :->import" \
&& ret=0

    case $state in
    (import)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-import-command-$line[1]:"
        case $line[1] in
            (records)
_arguments "${_arguments_options[@]}" : \
'--format=[]:FORMAT:(searchright-json json-lines csl-json ris nbib csv)' \
'--source-receipt-id=[]:SOURCE_RECEIPT_ID:_default' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(export-records)
_arguments "${_arguments_options[@]}" : \
'--review-id=[]:REVIEW_ID:_default' \
'--input-format=[]:INPUT_FORMAT:(searchright-json json-lines csl-json ris nbib csv)' \
'--output-format=[]:OUTPUT_FORMAT:(searchright-json json-lines csl-json ris nbib csv)' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(deduplicate)
_arguments "${_arguments_options[@]}" : \
'--title-threshold=[]:TITLE_THRESHOLD:_default' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__import__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-import-help-command-$line[1]:"
        case $line[1] in
            (records)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(export-records)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(deduplicate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(screen)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
":: :_searchright__subcmd__screen_commands" \
"*::: :->screen" \
&& ret=0

    case $state in
    (screen)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-screen-command-$line[1]:"
        case $line[1] in
            (rank)
_arguments "${_arguments_options[@]}" : \
'*--query-term=[]:QUERY_TERM:_default' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(study-graph)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-ranking-calibration)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(living-diff)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':previous:_files' \
':current:_files' \
&& ret=0
;;
(validate-living-lineage)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__screen__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-screen-help-command-$line[1]:"
        case $line[1] in
            (rank)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(study-graph)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-ranking-calibration)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(living-diff)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-living-lineage)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(report)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
":: :_searchright__subcmd__report_commands" \
"*::: :->report" \
&& ret=0

    case $state in
    (report)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-report-command-$line[1]:"
        case $line[1] in
            (prisma)
_arguments "${_arguments_options[@]}" : \
'--format=[]:FORMAT:(json mermaid ledger)' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(provenance)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(render-diagnostics)
_arguments "${_arguments_options[@]}" : \
'--format=[]:FORMAT:(plain-text json json-lines)' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__report__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-report-help-command-$line[1]:"
        case $line[1] in
            (prisma)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(provenance)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(render-diagnostics)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
;;
(completions)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':shell:(bash elvish fish powershell zsh)' \
&& ret=0
;;
(manpage)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(validate-plan)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-strategy)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-document-evidence)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(compile)
_arguments "${_arguments_options[@]}" : \
'--dialect=[]:DIALECT:(pubmed ovid-medline embase europe-pmc cinahl-ebsco psycinfo-ovid scopus web-of-science crossref openalex clinicaltrials-gov generic-boolean)' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(deduplicate)
_arguments "${_arguments_options[@]}" : \
'--title-threshold=[]:TITLE_THRESHOLD:_default' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(prisma)
_arguments "${_arguments_options[@]}" : \
'--format=[]:FORMAT:(json mermaid ledger)' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(verify-audit)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(import-records)
_arguments "${_arguments_options[@]}" : \
'--format=[]:FORMAT:(searchright-json json-lines csl-json ris nbib csv)' \
'--source-receipt-id=[]:SOURCE_RECEIPT_ID:_default' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(export-records)
_arguments "${_arguments_options[@]}" : \
'--review-id=[]:REVIEW_ID:_default' \
'--input-format=[]:INPUT_FORMAT:(searchright-json json-lines csl-json ris nbib csv)' \
'--output-format=[]:OUTPUT_FORMAT:(searchright-json json-lines csl-json ris nbib csv)' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(study-graph)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-search)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(living-diff)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':previous:_files' \
':current:_files' \
&& ret=0
;;
(validate-living-lineage)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(provenance)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(rank)
_arguments "${_arguments_options[@]}" : \
'*--query-term=[]:QUERY_TERM:_default' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(inspect-content)
_arguments "${_arguments_options[@]}" : \
'--subject-id=[]:SUBJECT_ID:_default' \
'--policy=[]:POLICY:(data-only sanitise-then-data-only human-inspection-required)' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(render-diagnostics)
_arguments "${_arguments_options[@]}" : \
'--format=[]:FORMAT:(plain-text json json-lines)' \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(evaluate-governance)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':policy:_files' \
':request:_files' \
&& ret=0
;;
(authorise-endpoint)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
':endpoint:_default' \
&& ret=0
;;
(validate-amendment)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-standard-pack)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-standard-assessment)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-ranking-calibration)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(validate-discovery-run)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(verify-workflow-trace)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(discovery-candidates)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(verify-provider-component)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':manifest:_files' \
':component:_files' \
&& ret=0
;;
(plan-licensed-request)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':profile:_files' \
':strategy:_files' \
':endpoint:_default' \
&& ret=0
;;
(validate-benchmark-report)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
':input:_files' \
&& ret=0
;;
(providers)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(workflow)
_arguments "${_arguments_options[@]}" : \
'-h[Print help]' \
'--help[Print help]' \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__help_commands" \
"*::: :->help" \
&& ret=0

    case $state in
    (help)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-help-command-$line[1]:"
        case $line[1] in
            (init)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(plan)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__help__subcmd__plan_commands" \
"*::: :->plan" \
&& ret=0

    case $state in
    (plan)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-help-plan-command-$line[1]:"
        case $line[1] in
            (validate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-amendment)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(evaluate-governance)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(workflow)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(source)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__help__subcmd__source_commands" \
"*::: :->source" \
&& ret=0

    case $state in
    (source)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-help-source-command-$line[1]:"
        case $line[1] in
            (list)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(authorise-endpoint)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-discovery-run)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(discovery-candidates)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(verify-provider-component)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(plan-licensed-request)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(strategy)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__help__subcmd__strategy_commands" \
"*::: :->strategy" \
&& ret=0

    case $state in
    (strategy)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-help-strategy-command-$line[1]:"
        case $line[1] in
            (validate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(compile)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-search)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-standard-pack)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-standard-assessment)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-benchmark-report)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(run)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__help__subcmd__run_commands" \
"*::: :->run" \
&& ret=0

    case $state in
    (run)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-help-run-command-$line[1]:"
        case $line[1] in
            (authorise-endpoint)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(verify-audit)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(verify-workflow-trace)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(inspect-content)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-document-evidence)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(import)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__help__subcmd__import_commands" \
"*::: :->import" \
&& ret=0

    case $state in
    (import)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-help-import-command-$line[1]:"
        case $line[1] in
            (records)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(export-records)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(deduplicate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(screen)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__help__subcmd__screen_commands" \
"*::: :->screen" \
&& ret=0

    case $state in
    (screen)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-help-screen-command-$line[1]:"
        case $line[1] in
            (rank)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(study-graph)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-ranking-calibration)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(living-diff)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-living-lineage)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(report)
_arguments "${_arguments_options[@]}" : \
":: :_searchright__subcmd__help__subcmd__report_commands" \
"*::: :->report" \
&& ret=0

    case $state in
    (report)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:searchright-help-report-command-$line[1]:"
        case $line[1] in
            (prisma)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(provenance)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(render-diagnostics)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
(completions)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(manpage)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-plan)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-strategy)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-document-evidence)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(compile)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(deduplicate)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(prisma)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(verify-audit)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(import-records)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(export-records)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(study-graph)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-search)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(living-diff)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-living-lineage)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(provenance)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(rank)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(inspect-content)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(render-diagnostics)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(evaluate-governance)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(authorise-endpoint)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-amendment)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-standard-pack)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-standard-assessment)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-ranking-calibration)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-discovery-run)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(verify-workflow-trace)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(discovery-candidates)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(verify-provider-component)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(plan-licensed-request)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(validate-benchmark-report)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(providers)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(workflow)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
(help)
_arguments "${_arguments_options[@]}" : \
&& ret=0
;;
        esac
    ;;
esac
;;
        esac
    ;;
esac
}

(( $+functions[_searchright_commands] )) ||
_searchright_commands() {
    local commands; commands=(
'init:Preview or apply creation of a conservative local CLI configuration' \
'plan:Review-plan operations' \
'source:Source and provider operations' \
'strategy:Search-strategy operations' \
'run:Bounded execution-authority operations' \
'import:Record import operations' \
'screen:Human-governed screening support operations' \
'report:Reporting and provenance operations' \
'completions:Generate a shell completion script on standard output' \
'manpage:Generate the searchright(1) manual page on standard output' \
'validate-plan:Validate a review plan and report readiness findings' \
'validate-strategy:Validate a source-specific search strategy' \
'validate-document-evidence:Validate neutral, non-canonical document extraction evidence' \
'compile:Compile a portable strategy into source syntax' \
'deduplicate:Deduplicate a JSON/YAML array of bibliographic records' \
'prisma:Validate or render a PRISMA flow contract' \
'verify-audit:Verify a JSONL hash-chained audit ledger' \
'import-records:Import bibliographic records and preserve source provenance' \
'export-records:Export canonical records with a conversion receipt' \
'study-graph:Validate and summarise a record-report-study graph' \
'validate-search:Evaluate PRESS, seed-set recall and translation-loss gates' \
'living-diff:Compare parent and current result sets for a living review' \
'validate-living-lineage:Validate a set of living-update lineage contracts' \
'provenance:Build RO-Crate and W3C PROV-compatible exports' \
'rank:Rank records transparently for prioritisation only' \
'inspect-content:Inspect untrusted text for instruction-like or active-content markers' \
'render-diagnostics:Render stable accessible diagnostics without ANSI-dependent output' \
'evaluate-governance:Evaluate a data-handling request against an institutional policy' \
'authorise-endpoint:Authorise an HTTPS endpoint against an execution envelope' \
'validate-amendment:Validate a protocol amendment' \
'validate-standard-pack:Validate a methodological standards pack' \
'validate-standard-assessment:Validate an assessment against a standards pack' \
'validate-ranking-calibration:Validate ranking calibration and its no-auto-exclusion contract' \
'validate-discovery-run:Validate a supplementary-discovery run' \
'verify-workflow-trace:Verify an evidence-bearing lifecycle trace against the finite assurance model' \
'discovery-candidates:Resolve bounded supplementary-discovery candidates for human release' \
'verify-provider-component:Verify a WASI provider-component manifest against exact component bytes' \
'plan-licensed-request:Build a redacted bring-your-own-access request plan' \
'validate-benchmark-report:Validate a benchmark report and its explicit claim boundary' \
'providers:List provider manifests available in the default no-network build' \
'workflow:Print the conservative agent workflow policy' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright commands' commands "$@"
}
(( $+functions[_searchright__subcmd__authorise-endpoint_commands] )) ||
_searchright__subcmd__authorise-endpoint_commands() {
    local commands; commands=()
    _describe -t commands 'searchright authorise-endpoint commands' commands "$@"
}
(( $+functions[_searchright__subcmd__compile_commands] )) ||
_searchright__subcmd__compile_commands() {
    local commands; commands=()
    _describe -t commands 'searchright compile commands' commands "$@"
}
(( $+functions[_searchright__subcmd__completions_commands] )) ||
_searchright__subcmd__completions_commands() {
    local commands; commands=()
    _describe -t commands 'searchright completions commands' commands "$@"
}
(( $+functions[_searchright__subcmd__deduplicate_commands] )) ||
_searchright__subcmd__deduplicate_commands() {
    local commands; commands=()
    _describe -t commands 'searchright deduplicate commands' commands "$@"
}
(( $+functions[_searchright__subcmd__discovery-candidates_commands] )) ||
_searchright__subcmd__discovery-candidates_commands() {
    local commands; commands=()
    _describe -t commands 'searchright discovery-candidates commands' commands "$@"
}
(( $+functions[_searchright__subcmd__evaluate-governance_commands] )) ||
_searchright__subcmd__evaluate-governance_commands() {
    local commands; commands=()
    _describe -t commands 'searchright evaluate-governance commands' commands "$@"
}
(( $+functions[_searchright__subcmd__export-records_commands] )) ||
_searchright__subcmd__export-records_commands() {
    local commands; commands=()
    _describe -t commands 'searchright export-records commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help_commands] )) ||
_searchright__subcmd__help_commands() {
    local commands; commands=(
'init:Preview or apply creation of a conservative local CLI configuration' \
'plan:Review-plan operations' \
'source:Source and provider operations' \
'strategy:Search-strategy operations' \
'run:Bounded execution-authority operations' \
'import:Record import operations' \
'screen:Human-governed screening support operations' \
'report:Reporting and provenance operations' \
'completions:Generate a shell completion script on standard output' \
'manpage:Generate the searchright(1) manual page on standard output' \
'validate-plan:Validate a review plan and report readiness findings' \
'validate-strategy:Validate a source-specific search strategy' \
'validate-document-evidence:Validate neutral, non-canonical document extraction evidence' \
'compile:Compile a portable strategy into source syntax' \
'deduplicate:Deduplicate a JSON/YAML array of bibliographic records' \
'prisma:Validate or render a PRISMA flow contract' \
'verify-audit:Verify a JSONL hash-chained audit ledger' \
'import-records:Import bibliographic records and preserve source provenance' \
'export-records:Export canonical records with a conversion receipt' \
'study-graph:Validate and summarise a record-report-study graph' \
'validate-search:Evaluate PRESS, seed-set recall and translation-loss gates' \
'living-diff:Compare parent and current result sets for a living review' \
'validate-living-lineage:Validate a set of living-update lineage contracts' \
'provenance:Build RO-Crate and W3C PROV-compatible exports' \
'rank:Rank records transparently for prioritisation only' \
'inspect-content:Inspect untrusted text for instruction-like or active-content markers' \
'render-diagnostics:Render stable accessible diagnostics without ANSI-dependent output' \
'evaluate-governance:Evaluate a data-handling request against an institutional policy' \
'authorise-endpoint:Authorise an HTTPS endpoint against an execution envelope' \
'validate-amendment:Validate a protocol amendment' \
'validate-standard-pack:Validate a methodological standards pack' \
'validate-standard-assessment:Validate an assessment against a standards pack' \
'validate-ranking-calibration:Validate ranking calibration and its no-auto-exclusion contract' \
'validate-discovery-run:Validate a supplementary-discovery run' \
'verify-workflow-trace:Verify an evidence-bearing lifecycle trace against the finite assurance model' \
'discovery-candidates:Resolve bounded supplementary-discovery candidates for human release' \
'verify-provider-component:Verify a WASI provider-component manifest against exact component bytes' \
'plan-licensed-request:Build a redacted bring-your-own-access request plan' \
'validate-benchmark-report:Validate a benchmark report and its explicit claim boundary' \
'providers:List provider manifests available in the default no-network build' \
'workflow:Print the conservative agent workflow policy' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__authorise-endpoint_commands] )) ||
_searchright__subcmd__help__subcmd__authorise-endpoint_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help authorise-endpoint commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__compile_commands] )) ||
_searchright__subcmd__help__subcmd__compile_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help compile commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__completions_commands] )) ||
_searchright__subcmd__help__subcmd__completions_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help completions commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__deduplicate_commands] )) ||
_searchright__subcmd__help__subcmd__deduplicate_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help deduplicate commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__discovery-candidates_commands] )) ||
_searchright__subcmd__help__subcmd__discovery-candidates_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help discovery-candidates commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__evaluate-governance_commands] )) ||
_searchright__subcmd__help__subcmd__evaluate-governance_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help evaluate-governance commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__export-records_commands] )) ||
_searchright__subcmd__help__subcmd__export-records_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help export-records commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__help_commands] )) ||
_searchright__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__import_commands] )) ||
_searchright__subcmd__help__subcmd__import_commands() {
    local commands; commands=(
'records:Import bibliographic records and preserve source provenance' \
'export-records:Export canonical records with a conversion receipt' \
'deduplicate:Deduplicate records without deleting source records' \
    )
    _describe -t commands 'searchright help import commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__import__subcmd__deduplicate_commands] )) ||
_searchright__subcmd__help__subcmd__import__subcmd__deduplicate_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help import deduplicate commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__import__subcmd__export-records_commands] )) ||
_searchright__subcmd__help__subcmd__import__subcmd__export-records_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help import export-records commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__import__subcmd__records_commands] )) ||
_searchright__subcmd__help__subcmd__import__subcmd__records_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help import records commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__import-records_commands] )) ||
_searchright__subcmd__help__subcmd__import-records_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help import-records commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__init_commands] )) ||
_searchright__subcmd__help__subcmd__init_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help init commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__inspect-content_commands] )) ||
_searchright__subcmd__help__subcmd__inspect-content_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help inspect-content commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__living-diff_commands] )) ||
_searchright__subcmd__help__subcmd__living-diff_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help living-diff commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__manpage_commands] )) ||
_searchright__subcmd__help__subcmd__manpage_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help manpage commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__plan_commands] )) ||
_searchright__subcmd__help__subcmd__plan_commands() {
    local commands; commands=(
'validate:Validate a review plan and report readiness findings' \
'validate-amendment:Validate a protocol amendment' \
'evaluate-governance:Evaluate a data-handling request against institutional policy' \
'workflow:Print the conservative agent workflow policy' \
    )
    _describe -t commands 'searchright help plan commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__plan__subcmd__evaluate-governance_commands] )) ||
_searchright__subcmd__help__subcmd__plan__subcmd__evaluate-governance_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help plan evaluate-governance commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__plan__subcmd__validate_commands] )) ||
_searchright__subcmd__help__subcmd__plan__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help plan validate commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__plan__subcmd__validate-amendment_commands] )) ||
_searchright__subcmd__help__subcmd__plan__subcmd__validate-amendment_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help plan validate-amendment commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__plan__subcmd__workflow_commands] )) ||
_searchright__subcmd__help__subcmd__plan__subcmd__workflow_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help plan workflow commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__plan-licensed-request_commands] )) ||
_searchright__subcmd__help__subcmd__plan-licensed-request_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help plan-licensed-request commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__prisma_commands] )) ||
_searchright__subcmd__help__subcmd__prisma_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help prisma commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__provenance_commands] )) ||
_searchright__subcmd__help__subcmd__provenance_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help provenance commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__providers_commands] )) ||
_searchright__subcmd__help__subcmd__providers_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help providers commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__rank_commands] )) ||
_searchright__subcmd__help__subcmd__rank_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help rank commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__render-diagnostics_commands] )) ||
_searchright__subcmd__help__subcmd__render-diagnostics_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help render-diagnostics commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__report_commands] )) ||
_searchright__subcmd__help__subcmd__report_commands() {
    local commands; commands=(
'prisma:Validate or render a PRISMA flow contract' \
'provenance:Build RO-Crate and W3C PROV-compatible exports' \
'render-diagnostics:Render stable accessible diagnostics without ANSI-dependent output' \
    )
    _describe -t commands 'searchright help report commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__report__subcmd__prisma_commands] )) ||
_searchright__subcmd__help__subcmd__report__subcmd__prisma_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help report prisma commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__report__subcmd__provenance_commands] )) ||
_searchright__subcmd__help__subcmd__report__subcmd__provenance_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help report provenance commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__report__subcmd__render-diagnostics_commands] )) ||
_searchright__subcmd__help__subcmd__report__subcmd__render-diagnostics_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help report render-diagnostics commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__run_commands] )) ||
_searchright__subcmd__help__subcmd__run_commands() {
    local commands; commands=(
'authorise-endpoint:Check endpoint authority without executing a request' \
'verify-audit:Verify a JSONL hash-chained audit ledger' \
'verify-workflow-trace:Verify an evidence-bearing lifecycle trace' \
'inspect-content:Inspect untrusted text without executing embedded instructions' \
'validate-document-evidence:Validate neutral, non-canonical document extraction evidence' \
    )
    _describe -t commands 'searchright help run commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__run__subcmd__authorise-endpoint_commands] )) ||
_searchright__subcmd__help__subcmd__run__subcmd__authorise-endpoint_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help run authorise-endpoint commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__run__subcmd__inspect-content_commands] )) ||
_searchright__subcmd__help__subcmd__run__subcmd__inspect-content_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help run inspect-content commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__run__subcmd__validate-document-evidence_commands] )) ||
_searchright__subcmd__help__subcmd__run__subcmd__validate-document-evidence_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help run validate-document-evidence commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__run__subcmd__verify-audit_commands] )) ||
_searchright__subcmd__help__subcmd__run__subcmd__verify-audit_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help run verify-audit commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__run__subcmd__verify-workflow-trace_commands] )) ||
_searchright__subcmd__help__subcmd__run__subcmd__verify-workflow-trace_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help run verify-workflow-trace commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__screen_commands] )) ||
_searchright__subcmd__help__subcmd__screen_commands() {
    local commands; commands=(
'rank:Rank records for prioritisation without making exclusion decisions' \
'study-graph:Validate and summarise explicit record-report-study linkage' \
'validate-ranking-calibration:Validate ranking calibration and its no-auto-exclusion contract' \
'living-diff:Compare parent and current result sets for a living review' \
'validate-living-lineage:Validate living-update lineage contracts' \
    )
    _describe -t commands 'searchright help screen commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__screen__subcmd__living-diff_commands] )) ||
_searchright__subcmd__help__subcmd__screen__subcmd__living-diff_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help screen living-diff commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__screen__subcmd__rank_commands] )) ||
_searchright__subcmd__help__subcmd__screen__subcmd__rank_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help screen rank commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__screen__subcmd__study-graph_commands] )) ||
_searchright__subcmd__help__subcmd__screen__subcmd__study-graph_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help screen study-graph commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__screen__subcmd__validate-living-lineage_commands] )) ||
_searchright__subcmd__help__subcmd__screen__subcmd__validate-living-lineage_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help screen validate-living-lineage commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__screen__subcmd__validate-ranking-calibration_commands] )) ||
_searchright__subcmd__help__subcmd__screen__subcmd__validate-ranking-calibration_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help screen validate-ranking-calibration commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__source_commands] )) ||
_searchright__subcmd__help__subcmd__source_commands() {
    local commands; commands=(
'list:List fixture-backed providers available without network access' \
'authorise-endpoint:Check endpoint authority without executing a request' \
'validate-discovery-run:Validate a supplementary-discovery run' \
'discovery-candidates:Resolve discovery candidates for human release' \
'verify-provider-component:Verify a WASI provider-component manifest against exact bytes' \
'plan-licensed-request:Build a redacted bring-your-own-access request plan' \
    )
    _describe -t commands 'searchright help source commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__source__subcmd__authorise-endpoint_commands] )) ||
_searchright__subcmd__help__subcmd__source__subcmd__authorise-endpoint_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help source authorise-endpoint commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__source__subcmd__discovery-candidates_commands] )) ||
_searchright__subcmd__help__subcmd__source__subcmd__discovery-candidates_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help source discovery-candidates commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__source__subcmd__list_commands] )) ||
_searchright__subcmd__help__subcmd__source__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help source list commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__source__subcmd__plan-licensed-request_commands] )) ||
_searchright__subcmd__help__subcmd__source__subcmd__plan-licensed-request_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help source plan-licensed-request commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__source__subcmd__validate-discovery-run_commands] )) ||
_searchright__subcmd__help__subcmd__source__subcmd__validate-discovery-run_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help source validate-discovery-run commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__source__subcmd__verify-provider-component_commands] )) ||
_searchright__subcmd__help__subcmd__source__subcmd__verify-provider-component_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help source verify-provider-component commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__strategy_commands] )) ||
_searchright__subcmd__help__subcmd__strategy_commands() {
    local commands; commands=(
'validate:Validate a source-specific search strategy' \
'compile:Compile a portable strategy into source syntax' \
'validate-search:Evaluate PRESS, seed-set recall and translation-loss gates' \
'validate-standard-pack:Validate a methodological standards pack' \
'validate-standard-assessment:Validate an assessment against a standards pack' \
'validate-benchmark-report:Validate a benchmark report and its explicit claim boundary' \
    )
    _describe -t commands 'searchright help strategy commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__strategy__subcmd__compile_commands] )) ||
_searchright__subcmd__help__subcmd__strategy__subcmd__compile_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help strategy compile commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__strategy__subcmd__validate_commands] )) ||
_searchright__subcmd__help__subcmd__strategy__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help strategy validate commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__strategy__subcmd__validate-benchmark-report_commands] )) ||
_searchright__subcmd__help__subcmd__strategy__subcmd__validate-benchmark-report_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help strategy validate-benchmark-report commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__strategy__subcmd__validate-search_commands] )) ||
_searchright__subcmd__help__subcmd__strategy__subcmd__validate-search_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help strategy validate-search commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__strategy__subcmd__validate-standard-assessment_commands] )) ||
_searchright__subcmd__help__subcmd__strategy__subcmd__validate-standard-assessment_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help strategy validate-standard-assessment commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__strategy__subcmd__validate-standard-pack_commands] )) ||
_searchright__subcmd__help__subcmd__strategy__subcmd__validate-standard-pack_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help strategy validate-standard-pack commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__study-graph_commands] )) ||
_searchright__subcmd__help__subcmd__study-graph_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help study-graph commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__validate-amendment_commands] )) ||
_searchright__subcmd__help__subcmd__validate-amendment_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help validate-amendment commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__validate-benchmark-report_commands] )) ||
_searchright__subcmd__help__subcmd__validate-benchmark-report_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help validate-benchmark-report commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__validate-discovery-run_commands] )) ||
_searchright__subcmd__help__subcmd__validate-discovery-run_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help validate-discovery-run commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__validate-document-evidence_commands] )) ||
_searchright__subcmd__help__subcmd__validate-document-evidence_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help validate-document-evidence commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__validate-living-lineage_commands] )) ||
_searchright__subcmd__help__subcmd__validate-living-lineage_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help validate-living-lineage commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__validate-plan_commands] )) ||
_searchright__subcmd__help__subcmd__validate-plan_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help validate-plan commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__validate-ranking-calibration_commands] )) ||
_searchright__subcmd__help__subcmd__validate-ranking-calibration_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help validate-ranking-calibration commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__validate-search_commands] )) ||
_searchright__subcmd__help__subcmd__validate-search_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help validate-search commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__validate-standard-assessment_commands] )) ||
_searchright__subcmd__help__subcmd__validate-standard-assessment_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help validate-standard-assessment commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__validate-standard-pack_commands] )) ||
_searchright__subcmd__help__subcmd__validate-standard-pack_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help validate-standard-pack commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__validate-strategy_commands] )) ||
_searchright__subcmd__help__subcmd__validate-strategy_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help validate-strategy commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__verify-audit_commands] )) ||
_searchright__subcmd__help__subcmd__verify-audit_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help verify-audit commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__verify-provider-component_commands] )) ||
_searchright__subcmd__help__subcmd__verify-provider-component_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help verify-provider-component commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__verify-workflow-trace_commands] )) ||
_searchright__subcmd__help__subcmd__verify-workflow-trace_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help verify-workflow-trace commands' commands "$@"
}
(( $+functions[_searchright__subcmd__help__subcmd__workflow_commands] )) ||
_searchright__subcmd__help__subcmd__workflow_commands() {
    local commands; commands=()
    _describe -t commands 'searchright help workflow commands' commands "$@"
}
(( $+functions[_searchright__subcmd__import_commands] )) ||
_searchright__subcmd__import_commands() {
    local commands; commands=(
'records:Import bibliographic records and preserve source provenance' \
'export-records:Export canonical records with a conversion receipt' \
'deduplicate:Deduplicate records without deleting source records' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright import commands' commands "$@"
}
(( $+functions[_searchright__subcmd__import__subcmd__deduplicate_commands] )) ||
_searchright__subcmd__import__subcmd__deduplicate_commands() {
    local commands; commands=()
    _describe -t commands 'searchright import deduplicate commands' commands "$@"
}
(( $+functions[_searchright__subcmd__import__subcmd__export-records_commands] )) ||
_searchright__subcmd__import__subcmd__export-records_commands() {
    local commands; commands=()
    _describe -t commands 'searchright import export-records commands' commands "$@"
}
(( $+functions[_searchright__subcmd__import__subcmd__help_commands] )) ||
_searchright__subcmd__import__subcmd__help_commands() {
    local commands; commands=(
'records:Import bibliographic records and preserve source provenance' \
'export-records:Export canonical records with a conversion receipt' \
'deduplicate:Deduplicate records without deleting source records' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright import help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__import__subcmd__help__subcmd__deduplicate_commands] )) ||
_searchright__subcmd__import__subcmd__help__subcmd__deduplicate_commands() {
    local commands; commands=()
    _describe -t commands 'searchright import help deduplicate commands' commands "$@"
}
(( $+functions[_searchright__subcmd__import__subcmd__help__subcmd__export-records_commands] )) ||
_searchright__subcmd__import__subcmd__help__subcmd__export-records_commands() {
    local commands; commands=()
    _describe -t commands 'searchright import help export-records commands' commands "$@"
}
(( $+functions[_searchright__subcmd__import__subcmd__help__subcmd__help_commands] )) ||
_searchright__subcmd__import__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'searchright import help help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__import__subcmd__help__subcmd__records_commands] )) ||
_searchright__subcmd__import__subcmd__help__subcmd__records_commands() {
    local commands; commands=()
    _describe -t commands 'searchright import help records commands' commands "$@"
}
(( $+functions[_searchright__subcmd__import__subcmd__records_commands] )) ||
_searchright__subcmd__import__subcmd__records_commands() {
    local commands; commands=()
    _describe -t commands 'searchright import records commands' commands "$@"
}
(( $+functions[_searchright__subcmd__import-records_commands] )) ||
_searchright__subcmd__import-records_commands() {
    local commands; commands=()
    _describe -t commands 'searchright import-records commands' commands "$@"
}
(( $+functions[_searchright__subcmd__init_commands] )) ||
_searchright__subcmd__init_commands() {
    local commands; commands=()
    _describe -t commands 'searchright init commands' commands "$@"
}
(( $+functions[_searchright__subcmd__inspect-content_commands] )) ||
_searchright__subcmd__inspect-content_commands() {
    local commands; commands=()
    _describe -t commands 'searchright inspect-content commands' commands "$@"
}
(( $+functions[_searchright__subcmd__living-diff_commands] )) ||
_searchright__subcmd__living-diff_commands() {
    local commands; commands=()
    _describe -t commands 'searchright living-diff commands' commands "$@"
}
(( $+functions[_searchright__subcmd__manpage_commands] )) ||
_searchright__subcmd__manpage_commands() {
    local commands; commands=()
    _describe -t commands 'searchright manpage commands' commands "$@"
}
(( $+functions[_searchright__subcmd__plan_commands] )) ||
_searchright__subcmd__plan_commands() {
    local commands; commands=(
'validate:Validate a review plan and report readiness findings' \
'validate-amendment:Validate a protocol amendment' \
'evaluate-governance:Evaluate a data-handling request against institutional policy' \
'workflow:Print the conservative agent workflow policy' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright plan commands' commands "$@"
}
(( $+functions[_searchright__subcmd__plan__subcmd__evaluate-governance_commands] )) ||
_searchright__subcmd__plan__subcmd__evaluate-governance_commands() {
    local commands; commands=()
    _describe -t commands 'searchright plan evaluate-governance commands' commands "$@"
}
(( $+functions[_searchright__subcmd__plan__subcmd__help_commands] )) ||
_searchright__subcmd__plan__subcmd__help_commands() {
    local commands; commands=(
'validate:Validate a review plan and report readiness findings' \
'validate-amendment:Validate a protocol amendment' \
'evaluate-governance:Evaluate a data-handling request against institutional policy' \
'workflow:Print the conservative agent workflow policy' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright plan help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__plan__subcmd__help__subcmd__evaluate-governance_commands] )) ||
_searchright__subcmd__plan__subcmd__help__subcmd__evaluate-governance_commands() {
    local commands; commands=()
    _describe -t commands 'searchright plan help evaluate-governance commands' commands "$@"
}
(( $+functions[_searchright__subcmd__plan__subcmd__help__subcmd__help_commands] )) ||
_searchright__subcmd__plan__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'searchright plan help help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__plan__subcmd__help__subcmd__validate_commands] )) ||
_searchright__subcmd__plan__subcmd__help__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'searchright plan help validate commands' commands "$@"
}
(( $+functions[_searchright__subcmd__plan__subcmd__help__subcmd__validate-amendment_commands] )) ||
_searchright__subcmd__plan__subcmd__help__subcmd__validate-amendment_commands() {
    local commands; commands=()
    _describe -t commands 'searchright plan help validate-amendment commands' commands "$@"
}
(( $+functions[_searchright__subcmd__plan__subcmd__help__subcmd__workflow_commands] )) ||
_searchright__subcmd__plan__subcmd__help__subcmd__workflow_commands() {
    local commands; commands=()
    _describe -t commands 'searchright plan help workflow commands' commands "$@"
}
(( $+functions[_searchright__subcmd__plan__subcmd__validate_commands] )) ||
_searchright__subcmd__plan__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'searchright plan validate commands' commands "$@"
}
(( $+functions[_searchright__subcmd__plan__subcmd__validate-amendment_commands] )) ||
_searchright__subcmd__plan__subcmd__validate-amendment_commands() {
    local commands; commands=()
    _describe -t commands 'searchright plan validate-amendment commands' commands "$@"
}
(( $+functions[_searchright__subcmd__plan__subcmd__workflow_commands] )) ||
_searchright__subcmd__plan__subcmd__workflow_commands() {
    local commands; commands=()
    _describe -t commands 'searchright plan workflow commands' commands "$@"
}
(( $+functions[_searchright__subcmd__plan-licensed-request_commands] )) ||
_searchright__subcmd__plan-licensed-request_commands() {
    local commands; commands=()
    _describe -t commands 'searchright plan-licensed-request commands' commands "$@"
}
(( $+functions[_searchright__subcmd__prisma_commands] )) ||
_searchright__subcmd__prisma_commands() {
    local commands; commands=()
    _describe -t commands 'searchright prisma commands' commands "$@"
}
(( $+functions[_searchright__subcmd__provenance_commands] )) ||
_searchright__subcmd__provenance_commands() {
    local commands; commands=()
    _describe -t commands 'searchright provenance commands' commands "$@"
}
(( $+functions[_searchright__subcmd__providers_commands] )) ||
_searchright__subcmd__providers_commands() {
    local commands; commands=()
    _describe -t commands 'searchright providers commands' commands "$@"
}
(( $+functions[_searchright__subcmd__rank_commands] )) ||
_searchright__subcmd__rank_commands() {
    local commands; commands=()
    _describe -t commands 'searchright rank commands' commands "$@"
}
(( $+functions[_searchright__subcmd__render-diagnostics_commands] )) ||
_searchright__subcmd__render-diagnostics_commands() {
    local commands; commands=()
    _describe -t commands 'searchright render-diagnostics commands' commands "$@"
}
(( $+functions[_searchright__subcmd__report_commands] )) ||
_searchright__subcmd__report_commands() {
    local commands; commands=(
'prisma:Validate or render a PRISMA flow contract' \
'provenance:Build RO-Crate and W3C PROV-compatible exports' \
'render-diagnostics:Render stable accessible diagnostics without ANSI-dependent output' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright report commands' commands "$@"
}
(( $+functions[_searchright__subcmd__report__subcmd__help_commands] )) ||
_searchright__subcmd__report__subcmd__help_commands() {
    local commands; commands=(
'prisma:Validate or render a PRISMA flow contract' \
'provenance:Build RO-Crate and W3C PROV-compatible exports' \
'render-diagnostics:Render stable accessible diagnostics without ANSI-dependent output' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright report help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__report__subcmd__help__subcmd__help_commands] )) ||
_searchright__subcmd__report__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'searchright report help help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__report__subcmd__help__subcmd__prisma_commands] )) ||
_searchright__subcmd__report__subcmd__help__subcmd__prisma_commands() {
    local commands; commands=()
    _describe -t commands 'searchright report help prisma commands' commands "$@"
}
(( $+functions[_searchright__subcmd__report__subcmd__help__subcmd__provenance_commands] )) ||
_searchright__subcmd__report__subcmd__help__subcmd__provenance_commands() {
    local commands; commands=()
    _describe -t commands 'searchright report help provenance commands' commands "$@"
}
(( $+functions[_searchright__subcmd__report__subcmd__help__subcmd__render-diagnostics_commands] )) ||
_searchright__subcmd__report__subcmd__help__subcmd__render-diagnostics_commands() {
    local commands; commands=()
    _describe -t commands 'searchright report help render-diagnostics commands' commands "$@"
}
(( $+functions[_searchright__subcmd__report__subcmd__prisma_commands] )) ||
_searchright__subcmd__report__subcmd__prisma_commands() {
    local commands; commands=()
    _describe -t commands 'searchright report prisma commands' commands "$@"
}
(( $+functions[_searchright__subcmd__report__subcmd__provenance_commands] )) ||
_searchright__subcmd__report__subcmd__provenance_commands() {
    local commands; commands=()
    _describe -t commands 'searchright report provenance commands' commands "$@"
}
(( $+functions[_searchright__subcmd__report__subcmd__render-diagnostics_commands] )) ||
_searchright__subcmd__report__subcmd__render-diagnostics_commands() {
    local commands; commands=()
    _describe -t commands 'searchright report render-diagnostics commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run_commands] )) ||
_searchright__subcmd__run_commands() {
    local commands; commands=(
'authorise-endpoint:Check endpoint authority without executing a request' \
'verify-audit:Verify a JSONL hash-chained audit ledger' \
'verify-workflow-trace:Verify an evidence-bearing lifecycle trace' \
'inspect-content:Inspect untrusted text without executing embedded instructions' \
'validate-document-evidence:Validate neutral, non-canonical document extraction evidence' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright run commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run__subcmd__authorise-endpoint_commands] )) ||
_searchright__subcmd__run__subcmd__authorise-endpoint_commands() {
    local commands; commands=()
    _describe -t commands 'searchright run authorise-endpoint commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run__subcmd__help_commands] )) ||
_searchright__subcmd__run__subcmd__help_commands() {
    local commands; commands=(
'authorise-endpoint:Check endpoint authority without executing a request' \
'verify-audit:Verify a JSONL hash-chained audit ledger' \
'verify-workflow-trace:Verify an evidence-bearing lifecycle trace' \
'inspect-content:Inspect untrusted text without executing embedded instructions' \
'validate-document-evidence:Validate neutral, non-canonical document extraction evidence' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright run help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run__subcmd__help__subcmd__authorise-endpoint_commands] )) ||
_searchright__subcmd__run__subcmd__help__subcmd__authorise-endpoint_commands() {
    local commands; commands=()
    _describe -t commands 'searchright run help authorise-endpoint commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run__subcmd__help__subcmd__help_commands] )) ||
_searchright__subcmd__run__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'searchright run help help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run__subcmd__help__subcmd__inspect-content_commands] )) ||
_searchright__subcmd__run__subcmd__help__subcmd__inspect-content_commands() {
    local commands; commands=()
    _describe -t commands 'searchright run help inspect-content commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run__subcmd__help__subcmd__validate-document-evidence_commands] )) ||
_searchright__subcmd__run__subcmd__help__subcmd__validate-document-evidence_commands() {
    local commands; commands=()
    _describe -t commands 'searchright run help validate-document-evidence commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run__subcmd__help__subcmd__verify-audit_commands] )) ||
_searchright__subcmd__run__subcmd__help__subcmd__verify-audit_commands() {
    local commands; commands=()
    _describe -t commands 'searchright run help verify-audit commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run__subcmd__help__subcmd__verify-workflow-trace_commands] )) ||
_searchright__subcmd__run__subcmd__help__subcmd__verify-workflow-trace_commands() {
    local commands; commands=()
    _describe -t commands 'searchright run help verify-workflow-trace commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run__subcmd__inspect-content_commands] )) ||
_searchright__subcmd__run__subcmd__inspect-content_commands() {
    local commands; commands=()
    _describe -t commands 'searchright run inspect-content commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run__subcmd__validate-document-evidence_commands] )) ||
_searchright__subcmd__run__subcmd__validate-document-evidence_commands() {
    local commands; commands=()
    _describe -t commands 'searchright run validate-document-evidence commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run__subcmd__verify-audit_commands] )) ||
_searchright__subcmd__run__subcmd__verify-audit_commands() {
    local commands; commands=()
    _describe -t commands 'searchright run verify-audit commands' commands "$@"
}
(( $+functions[_searchright__subcmd__run__subcmd__verify-workflow-trace_commands] )) ||
_searchright__subcmd__run__subcmd__verify-workflow-trace_commands() {
    local commands; commands=()
    _describe -t commands 'searchright run verify-workflow-trace commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen_commands] )) ||
_searchright__subcmd__screen_commands() {
    local commands; commands=(
'rank:Rank records for prioritisation without making exclusion decisions' \
'study-graph:Validate and summarise explicit record-report-study linkage' \
'validate-ranking-calibration:Validate ranking calibration and its no-auto-exclusion contract' \
'living-diff:Compare parent and current result sets for a living review' \
'validate-living-lineage:Validate living-update lineage contracts' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright screen commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen__subcmd__help_commands] )) ||
_searchright__subcmd__screen__subcmd__help_commands() {
    local commands; commands=(
'rank:Rank records for prioritisation without making exclusion decisions' \
'study-graph:Validate and summarise explicit record-report-study linkage' \
'validate-ranking-calibration:Validate ranking calibration and its no-auto-exclusion contract' \
'living-diff:Compare parent and current result sets for a living review' \
'validate-living-lineage:Validate living-update lineage contracts' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright screen help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen__subcmd__help__subcmd__help_commands] )) ||
_searchright__subcmd__screen__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'searchright screen help help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen__subcmd__help__subcmd__living-diff_commands] )) ||
_searchright__subcmd__screen__subcmd__help__subcmd__living-diff_commands() {
    local commands; commands=()
    _describe -t commands 'searchright screen help living-diff commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen__subcmd__help__subcmd__rank_commands] )) ||
_searchright__subcmd__screen__subcmd__help__subcmd__rank_commands() {
    local commands; commands=()
    _describe -t commands 'searchright screen help rank commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen__subcmd__help__subcmd__study-graph_commands] )) ||
_searchright__subcmd__screen__subcmd__help__subcmd__study-graph_commands() {
    local commands; commands=()
    _describe -t commands 'searchright screen help study-graph commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen__subcmd__help__subcmd__validate-living-lineage_commands] )) ||
_searchright__subcmd__screen__subcmd__help__subcmd__validate-living-lineage_commands() {
    local commands; commands=()
    _describe -t commands 'searchright screen help validate-living-lineage commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen__subcmd__help__subcmd__validate-ranking-calibration_commands] )) ||
_searchright__subcmd__screen__subcmd__help__subcmd__validate-ranking-calibration_commands() {
    local commands; commands=()
    _describe -t commands 'searchright screen help validate-ranking-calibration commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen__subcmd__living-diff_commands] )) ||
_searchright__subcmd__screen__subcmd__living-diff_commands() {
    local commands; commands=()
    _describe -t commands 'searchright screen living-diff commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen__subcmd__rank_commands] )) ||
_searchright__subcmd__screen__subcmd__rank_commands() {
    local commands; commands=()
    _describe -t commands 'searchright screen rank commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen__subcmd__study-graph_commands] )) ||
_searchright__subcmd__screen__subcmd__study-graph_commands() {
    local commands; commands=()
    _describe -t commands 'searchright screen study-graph commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen__subcmd__validate-living-lineage_commands] )) ||
_searchright__subcmd__screen__subcmd__validate-living-lineage_commands() {
    local commands; commands=()
    _describe -t commands 'searchright screen validate-living-lineage commands' commands "$@"
}
(( $+functions[_searchright__subcmd__screen__subcmd__validate-ranking-calibration_commands] )) ||
_searchright__subcmd__screen__subcmd__validate-ranking-calibration_commands() {
    local commands; commands=()
    _describe -t commands 'searchright screen validate-ranking-calibration commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source_commands] )) ||
_searchright__subcmd__source_commands() {
    local commands; commands=(
'list:List fixture-backed providers available without network access' \
'authorise-endpoint:Check endpoint authority without executing a request' \
'validate-discovery-run:Validate a supplementary-discovery run' \
'discovery-candidates:Resolve discovery candidates for human release' \
'verify-provider-component:Verify a WASI provider-component manifest against exact bytes' \
'plan-licensed-request:Build a redacted bring-your-own-access request plan' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright source commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__authorise-endpoint_commands] )) ||
_searchright__subcmd__source__subcmd__authorise-endpoint_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source authorise-endpoint commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__discovery-candidates_commands] )) ||
_searchright__subcmd__source__subcmd__discovery-candidates_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source discovery-candidates commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__help_commands] )) ||
_searchright__subcmd__source__subcmd__help_commands() {
    local commands; commands=(
'list:List fixture-backed providers available without network access' \
'authorise-endpoint:Check endpoint authority without executing a request' \
'validate-discovery-run:Validate a supplementary-discovery run' \
'discovery-candidates:Resolve discovery candidates for human release' \
'verify-provider-component:Verify a WASI provider-component manifest against exact bytes' \
'plan-licensed-request:Build a redacted bring-your-own-access request plan' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright source help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__help__subcmd__authorise-endpoint_commands] )) ||
_searchright__subcmd__source__subcmd__help__subcmd__authorise-endpoint_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source help authorise-endpoint commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__help__subcmd__discovery-candidates_commands] )) ||
_searchright__subcmd__source__subcmd__help__subcmd__discovery-candidates_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source help discovery-candidates commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__help__subcmd__help_commands] )) ||
_searchright__subcmd__source__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source help help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__help__subcmd__list_commands] )) ||
_searchright__subcmd__source__subcmd__help__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source help list commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__help__subcmd__plan-licensed-request_commands] )) ||
_searchright__subcmd__source__subcmd__help__subcmd__plan-licensed-request_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source help plan-licensed-request commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__help__subcmd__validate-discovery-run_commands] )) ||
_searchright__subcmd__source__subcmd__help__subcmd__validate-discovery-run_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source help validate-discovery-run commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__help__subcmd__verify-provider-component_commands] )) ||
_searchright__subcmd__source__subcmd__help__subcmd__verify-provider-component_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source help verify-provider-component commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__list_commands] )) ||
_searchright__subcmd__source__subcmd__list_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source list commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__plan-licensed-request_commands] )) ||
_searchright__subcmd__source__subcmd__plan-licensed-request_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source plan-licensed-request commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__validate-discovery-run_commands] )) ||
_searchright__subcmd__source__subcmd__validate-discovery-run_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source validate-discovery-run commands' commands "$@"
}
(( $+functions[_searchright__subcmd__source__subcmd__verify-provider-component_commands] )) ||
_searchright__subcmd__source__subcmd__verify-provider-component_commands() {
    local commands; commands=()
    _describe -t commands 'searchright source verify-provider-component commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy_commands] )) ||
_searchright__subcmd__strategy_commands() {
    local commands; commands=(
'validate:Validate a source-specific search strategy' \
'compile:Compile a portable strategy into source syntax' \
'validate-search:Evaluate PRESS, seed-set recall and translation-loss gates' \
'validate-standard-pack:Validate a methodological standards pack' \
'validate-standard-assessment:Validate an assessment against a standards pack' \
'validate-benchmark-report:Validate a benchmark report and its explicit claim boundary' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright strategy commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__compile_commands] )) ||
_searchright__subcmd__strategy__subcmd__compile_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy compile commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__help_commands] )) ||
_searchright__subcmd__strategy__subcmd__help_commands() {
    local commands; commands=(
'validate:Validate a source-specific search strategy' \
'compile:Compile a portable strategy into source syntax' \
'validate-search:Evaluate PRESS, seed-set recall and translation-loss gates' \
'validate-standard-pack:Validate a methodological standards pack' \
'validate-standard-assessment:Validate an assessment against a standards pack' \
'validate-benchmark-report:Validate a benchmark report and its explicit claim boundary' \
'help:Print this message or the help of the given subcommand(s)' \
    )
    _describe -t commands 'searchright strategy help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__help__subcmd__compile_commands] )) ||
_searchright__subcmd__strategy__subcmd__help__subcmd__compile_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy help compile commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__help__subcmd__help_commands] )) ||
_searchright__subcmd__strategy__subcmd__help__subcmd__help_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy help help commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__help__subcmd__validate_commands] )) ||
_searchright__subcmd__strategy__subcmd__help__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy help validate commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__help__subcmd__validate-benchmark-report_commands] )) ||
_searchright__subcmd__strategy__subcmd__help__subcmd__validate-benchmark-report_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy help validate-benchmark-report commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__help__subcmd__validate-search_commands] )) ||
_searchright__subcmd__strategy__subcmd__help__subcmd__validate-search_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy help validate-search commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__help__subcmd__validate-standard-assessment_commands] )) ||
_searchright__subcmd__strategy__subcmd__help__subcmd__validate-standard-assessment_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy help validate-standard-assessment commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__help__subcmd__validate-standard-pack_commands] )) ||
_searchright__subcmd__strategy__subcmd__help__subcmd__validate-standard-pack_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy help validate-standard-pack commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__validate_commands] )) ||
_searchright__subcmd__strategy__subcmd__validate_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy validate commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__validate-benchmark-report_commands] )) ||
_searchright__subcmd__strategy__subcmd__validate-benchmark-report_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy validate-benchmark-report commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__validate-search_commands] )) ||
_searchright__subcmd__strategy__subcmd__validate-search_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy validate-search commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__validate-standard-assessment_commands] )) ||
_searchright__subcmd__strategy__subcmd__validate-standard-assessment_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy validate-standard-assessment commands' commands "$@"
}
(( $+functions[_searchright__subcmd__strategy__subcmd__validate-standard-pack_commands] )) ||
_searchright__subcmd__strategy__subcmd__validate-standard-pack_commands() {
    local commands; commands=()
    _describe -t commands 'searchright strategy validate-standard-pack commands' commands "$@"
}
(( $+functions[_searchright__subcmd__study-graph_commands] )) ||
_searchright__subcmd__study-graph_commands() {
    local commands; commands=()
    _describe -t commands 'searchright study-graph commands' commands "$@"
}
(( $+functions[_searchright__subcmd__validate-amendment_commands] )) ||
_searchright__subcmd__validate-amendment_commands() {
    local commands; commands=()
    _describe -t commands 'searchright validate-amendment commands' commands "$@"
}
(( $+functions[_searchright__subcmd__validate-benchmark-report_commands] )) ||
_searchright__subcmd__validate-benchmark-report_commands() {
    local commands; commands=()
    _describe -t commands 'searchright validate-benchmark-report commands' commands "$@"
}
(( $+functions[_searchright__subcmd__validate-discovery-run_commands] )) ||
_searchright__subcmd__validate-discovery-run_commands() {
    local commands; commands=()
    _describe -t commands 'searchright validate-discovery-run commands' commands "$@"
}
(( $+functions[_searchright__subcmd__validate-document-evidence_commands] )) ||
_searchright__subcmd__validate-document-evidence_commands() {
    local commands; commands=()
    _describe -t commands 'searchright validate-document-evidence commands' commands "$@"
}
(( $+functions[_searchright__subcmd__validate-living-lineage_commands] )) ||
_searchright__subcmd__validate-living-lineage_commands() {
    local commands; commands=()
    _describe -t commands 'searchright validate-living-lineage commands' commands "$@"
}
(( $+functions[_searchright__subcmd__validate-plan_commands] )) ||
_searchright__subcmd__validate-plan_commands() {
    local commands; commands=()
    _describe -t commands 'searchright validate-plan commands' commands "$@"
}
(( $+functions[_searchright__subcmd__validate-ranking-calibration_commands] )) ||
_searchright__subcmd__validate-ranking-calibration_commands() {
    local commands; commands=()
    _describe -t commands 'searchright validate-ranking-calibration commands' commands "$@"
}
(( $+functions[_searchright__subcmd__validate-search_commands] )) ||
_searchright__subcmd__validate-search_commands() {
    local commands; commands=()
    _describe -t commands 'searchright validate-search commands' commands "$@"
}
(( $+functions[_searchright__subcmd__validate-standard-assessment_commands] )) ||
_searchright__subcmd__validate-standard-assessment_commands() {
    local commands; commands=()
    _describe -t commands 'searchright validate-standard-assessment commands' commands "$@"
}
(( $+functions[_searchright__subcmd__validate-standard-pack_commands] )) ||
_searchright__subcmd__validate-standard-pack_commands() {
    local commands; commands=()
    _describe -t commands 'searchright validate-standard-pack commands' commands "$@"
}
(( $+functions[_searchright__subcmd__validate-strategy_commands] )) ||
_searchright__subcmd__validate-strategy_commands() {
    local commands; commands=()
    _describe -t commands 'searchright validate-strategy commands' commands "$@"
}
(( $+functions[_searchright__subcmd__verify-audit_commands] )) ||
_searchright__subcmd__verify-audit_commands() {
    local commands; commands=()
    _describe -t commands 'searchright verify-audit commands' commands "$@"
}
(( $+functions[_searchright__subcmd__verify-provider-component_commands] )) ||
_searchright__subcmd__verify-provider-component_commands() {
    local commands; commands=()
    _describe -t commands 'searchright verify-provider-component commands' commands "$@"
}
(( $+functions[_searchright__subcmd__verify-workflow-trace_commands] )) ||
_searchright__subcmd__verify-workflow-trace_commands() {
    local commands; commands=()
    _describe -t commands 'searchright verify-workflow-trace commands' commands "$@"
}
(( $+functions[_searchright__subcmd__workflow_commands] )) ||
_searchright__subcmd__workflow_commands() {
    local commands; commands=()
    _describe -t commands 'searchright workflow commands' commands "$@"
}

if [ "$funcstack[1]" = "_searchright" ]; then
    _searchright "$@"
else
    compdef _searchright searchright
fi
