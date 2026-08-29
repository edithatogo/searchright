_searchright() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="searchright"
                ;;
            searchright,authorise-endpoint)
                cmd="searchright__subcmd__authorise__subcmd__endpoint"
                ;;
            searchright,compile)
                cmd="searchright__subcmd__compile"
                ;;
            searchright,completions)
                cmd="searchright__subcmd__completions"
                ;;
            searchright,deduplicate)
                cmd="searchright__subcmd__deduplicate"
                ;;
            searchright,discovery-candidates)
                cmd="searchright__subcmd__discovery__subcmd__candidates"
                ;;
            searchright,evaluate-governance)
                cmd="searchright__subcmd__evaluate__subcmd__governance"
                ;;
            searchright,export-records)
                cmd="searchright__subcmd__export__subcmd__records"
                ;;
            searchright,help)
                cmd="searchright__subcmd__help"
                ;;
            searchright,import)
                cmd="searchright__subcmd__import"
                ;;
            searchright,import-records)
                cmd="searchright__subcmd__import__subcmd__records"
                ;;
            searchright,init)
                cmd="searchright__subcmd__init"
                ;;
            searchright,inspect-content)
                cmd="searchright__subcmd__inspect__subcmd__content"
                ;;
            searchright,living-diff)
                cmd="searchright__subcmd__living__subcmd__diff"
                ;;
            searchright,manpage)
                cmd="searchright__subcmd__manpage"
                ;;
            searchright,plan)
                cmd="searchright__subcmd__plan"
                ;;
            searchright,plan-licensed-request)
                cmd="searchright__subcmd__plan__subcmd__licensed__subcmd__request"
                ;;
            searchright,prisma)
                cmd="searchright__subcmd__prisma"
                ;;
            searchright,provenance)
                cmd="searchright__subcmd__provenance"
                ;;
            searchright,providers)
                cmd="searchright__subcmd__providers"
                ;;
            searchright,rank)
                cmd="searchright__subcmd__rank"
                ;;
            searchright,render-diagnostics)
                cmd="searchright__subcmd__render__subcmd__diagnostics"
                ;;
            searchright,report)
                cmd="searchright__subcmd__report"
                ;;
            searchright,run)
                cmd="searchright__subcmd__run"
                ;;
            searchright,screen)
                cmd="searchright__subcmd__screen"
                ;;
            searchright,source)
                cmd="searchright__subcmd__source"
                ;;
            searchright,strategy)
                cmd="searchright__subcmd__strategy"
                ;;
            searchright,study-graph)
                cmd="searchright__subcmd__study__subcmd__graph"
                ;;
            searchright,validate-amendment)
                cmd="searchright__subcmd__validate__subcmd__amendment"
                ;;
            searchright,validate-benchmark-report)
                cmd="searchright__subcmd__validate__subcmd__benchmark__subcmd__report"
                ;;
            searchright,validate-discovery-run)
                cmd="searchright__subcmd__validate__subcmd__discovery__subcmd__run"
                ;;
            searchright,validate-document-evidence)
                cmd="searchright__subcmd__validate__subcmd__document__subcmd__evidence"
                ;;
            searchright,validate-living-lineage)
                cmd="searchright__subcmd__validate__subcmd__living__subcmd__lineage"
                ;;
            searchright,validate-plan)
                cmd="searchright__subcmd__validate__subcmd__plan"
                ;;
            searchright,validate-ranking-calibration)
                cmd="searchright__subcmd__validate__subcmd__ranking__subcmd__calibration"
                ;;
            searchright,validate-search)
                cmd="searchright__subcmd__validate__subcmd__search"
                ;;
            searchright,validate-standard-assessment)
                cmd="searchright__subcmd__validate__subcmd__standard__subcmd__assessment"
                ;;
            searchright,validate-standard-pack)
                cmd="searchright__subcmd__validate__subcmd__standard__subcmd__pack"
                ;;
            searchright,validate-strategy)
                cmd="searchright__subcmd__validate__subcmd__strategy"
                ;;
            searchright,verify-audit)
                cmd="searchright__subcmd__verify__subcmd__audit"
                ;;
            searchright,verify-provider-component)
                cmd="searchright__subcmd__verify__subcmd__provider__subcmd__component"
                ;;
            searchright,verify-workflow-trace)
                cmd="searchright__subcmd__verify__subcmd__workflow__subcmd__trace"
                ;;
            searchright,workflow)
                cmd="searchright__subcmd__workflow"
                ;;
            searchright__subcmd__help,authorise-endpoint)
                cmd="searchright__subcmd__help__subcmd__authorise__subcmd__endpoint"
                ;;
            searchright__subcmd__help,compile)
                cmd="searchright__subcmd__help__subcmd__compile"
                ;;
            searchright__subcmd__help,completions)
                cmd="searchright__subcmd__help__subcmd__completions"
                ;;
            searchright__subcmd__help,deduplicate)
                cmd="searchright__subcmd__help__subcmd__deduplicate"
                ;;
            searchright__subcmd__help,discovery-candidates)
                cmd="searchright__subcmd__help__subcmd__discovery__subcmd__candidates"
                ;;
            searchright__subcmd__help,evaluate-governance)
                cmd="searchright__subcmd__help__subcmd__evaluate__subcmd__governance"
                ;;
            searchright__subcmd__help,export-records)
                cmd="searchright__subcmd__help__subcmd__export__subcmd__records"
                ;;
            searchright__subcmd__help,help)
                cmd="searchright__subcmd__help__subcmd__help"
                ;;
            searchright__subcmd__help,import)
                cmd="searchright__subcmd__help__subcmd__import"
                ;;
            searchright__subcmd__help,import-records)
                cmd="searchright__subcmd__help__subcmd__import__subcmd__records"
                ;;
            searchright__subcmd__help,init)
                cmd="searchright__subcmd__help__subcmd__init"
                ;;
            searchright__subcmd__help,inspect-content)
                cmd="searchright__subcmd__help__subcmd__inspect__subcmd__content"
                ;;
            searchright__subcmd__help,living-diff)
                cmd="searchright__subcmd__help__subcmd__living__subcmd__diff"
                ;;
            searchright__subcmd__help,manpage)
                cmd="searchright__subcmd__help__subcmd__manpage"
                ;;
            searchright__subcmd__help,plan)
                cmd="searchright__subcmd__help__subcmd__plan"
                ;;
            searchright__subcmd__help,plan-licensed-request)
                cmd="searchright__subcmd__help__subcmd__plan__subcmd__licensed__subcmd__request"
                ;;
            searchright__subcmd__help,prisma)
                cmd="searchright__subcmd__help__subcmd__prisma"
                ;;
            searchright__subcmd__help,provenance)
                cmd="searchright__subcmd__help__subcmd__provenance"
                ;;
            searchright__subcmd__help,providers)
                cmd="searchright__subcmd__help__subcmd__providers"
                ;;
            searchright__subcmd__help,rank)
                cmd="searchright__subcmd__help__subcmd__rank"
                ;;
            searchright__subcmd__help,render-diagnostics)
                cmd="searchright__subcmd__help__subcmd__render__subcmd__diagnostics"
                ;;
            searchright__subcmd__help,report)
                cmd="searchright__subcmd__help__subcmd__report"
                ;;
            searchright__subcmd__help,run)
                cmd="searchright__subcmd__help__subcmd__run"
                ;;
            searchright__subcmd__help,screen)
                cmd="searchright__subcmd__help__subcmd__screen"
                ;;
            searchright__subcmd__help,source)
                cmd="searchright__subcmd__help__subcmd__source"
                ;;
            searchright__subcmd__help,strategy)
                cmd="searchright__subcmd__help__subcmd__strategy"
                ;;
            searchright__subcmd__help,study-graph)
                cmd="searchright__subcmd__help__subcmd__study__subcmd__graph"
                ;;
            searchright__subcmd__help,validate-amendment)
                cmd="searchright__subcmd__help__subcmd__validate__subcmd__amendment"
                ;;
            searchright__subcmd__help,validate-benchmark-report)
                cmd="searchright__subcmd__help__subcmd__validate__subcmd__benchmark__subcmd__report"
                ;;
            searchright__subcmd__help,validate-discovery-run)
                cmd="searchright__subcmd__help__subcmd__validate__subcmd__discovery__subcmd__run"
                ;;
            searchright__subcmd__help,validate-document-evidence)
                cmd="searchright__subcmd__help__subcmd__validate__subcmd__document__subcmd__evidence"
                ;;
            searchright__subcmd__help,validate-living-lineage)
                cmd="searchright__subcmd__help__subcmd__validate__subcmd__living__subcmd__lineage"
                ;;
            searchright__subcmd__help,validate-plan)
                cmd="searchright__subcmd__help__subcmd__validate__subcmd__plan"
                ;;
            searchright__subcmd__help,validate-ranking-calibration)
                cmd="searchright__subcmd__help__subcmd__validate__subcmd__ranking__subcmd__calibration"
                ;;
            searchright__subcmd__help,validate-search)
                cmd="searchright__subcmd__help__subcmd__validate__subcmd__search"
                ;;
            searchright__subcmd__help,validate-standard-assessment)
                cmd="searchright__subcmd__help__subcmd__validate__subcmd__standard__subcmd__assessment"
                ;;
            searchright__subcmd__help,validate-standard-pack)
                cmd="searchright__subcmd__help__subcmd__validate__subcmd__standard__subcmd__pack"
                ;;
            searchright__subcmd__help,validate-strategy)
                cmd="searchright__subcmd__help__subcmd__validate__subcmd__strategy"
                ;;
            searchright__subcmd__help,verify-audit)
                cmd="searchright__subcmd__help__subcmd__verify__subcmd__audit"
                ;;
            searchright__subcmd__help,verify-provider-component)
                cmd="searchright__subcmd__help__subcmd__verify__subcmd__provider__subcmd__component"
                ;;
            searchright__subcmd__help,verify-workflow-trace)
                cmd="searchright__subcmd__help__subcmd__verify__subcmd__workflow__subcmd__trace"
                ;;
            searchright__subcmd__help,workflow)
                cmd="searchright__subcmd__help__subcmd__workflow"
                ;;
            searchright__subcmd__help__subcmd__import,deduplicate)
                cmd="searchright__subcmd__help__subcmd__import__subcmd__deduplicate"
                ;;
            searchright__subcmd__help__subcmd__import,export-records)
                cmd="searchright__subcmd__help__subcmd__import__subcmd__export__subcmd__records"
                ;;
            searchright__subcmd__help__subcmd__import,records)
                cmd="searchright__subcmd__help__subcmd__import__subcmd__records"
                ;;
            searchright__subcmd__help__subcmd__plan,evaluate-governance)
                cmd="searchright__subcmd__help__subcmd__plan__subcmd__evaluate__subcmd__governance"
                ;;
            searchright__subcmd__help__subcmd__plan,validate)
                cmd="searchright__subcmd__help__subcmd__plan__subcmd__validate"
                ;;
            searchright__subcmd__help__subcmd__plan,validate-amendment)
                cmd="searchright__subcmd__help__subcmd__plan__subcmd__validate__subcmd__amendment"
                ;;
            searchright__subcmd__help__subcmd__plan,workflow)
                cmd="searchright__subcmd__help__subcmd__plan__subcmd__workflow"
                ;;
            searchright__subcmd__help__subcmd__report,prisma)
                cmd="searchright__subcmd__help__subcmd__report__subcmd__prisma"
                ;;
            searchright__subcmd__help__subcmd__report,provenance)
                cmd="searchright__subcmd__help__subcmd__report__subcmd__provenance"
                ;;
            searchright__subcmd__help__subcmd__report,render-diagnostics)
                cmd="searchright__subcmd__help__subcmd__report__subcmd__render__subcmd__diagnostics"
                ;;
            searchright__subcmd__help__subcmd__run,authorise-endpoint)
                cmd="searchright__subcmd__help__subcmd__run__subcmd__authorise__subcmd__endpoint"
                ;;
            searchright__subcmd__help__subcmd__run,inspect-content)
                cmd="searchright__subcmd__help__subcmd__run__subcmd__inspect__subcmd__content"
                ;;
            searchright__subcmd__help__subcmd__run,validate-document-evidence)
                cmd="searchright__subcmd__help__subcmd__run__subcmd__validate__subcmd__document__subcmd__evidence"
                ;;
            searchright__subcmd__help__subcmd__run,verify-audit)
                cmd="searchright__subcmd__help__subcmd__run__subcmd__verify__subcmd__audit"
                ;;
            searchright__subcmd__help__subcmd__run,verify-workflow-trace)
                cmd="searchright__subcmd__help__subcmd__run__subcmd__verify__subcmd__workflow__subcmd__trace"
                ;;
            searchright__subcmd__help__subcmd__screen,living-diff)
                cmd="searchright__subcmd__help__subcmd__screen__subcmd__living__subcmd__diff"
                ;;
            searchright__subcmd__help__subcmd__screen,rank)
                cmd="searchright__subcmd__help__subcmd__screen__subcmd__rank"
                ;;
            searchright__subcmd__help__subcmd__screen,study-graph)
                cmd="searchright__subcmd__help__subcmd__screen__subcmd__study__subcmd__graph"
                ;;
            searchright__subcmd__help__subcmd__screen,validate-living-lineage)
                cmd="searchright__subcmd__help__subcmd__screen__subcmd__validate__subcmd__living__subcmd__lineage"
                ;;
            searchright__subcmd__help__subcmd__screen,validate-ranking-calibration)
                cmd="searchright__subcmd__help__subcmd__screen__subcmd__validate__subcmd__ranking__subcmd__calibration"
                ;;
            searchright__subcmd__help__subcmd__source,authorise-endpoint)
                cmd="searchright__subcmd__help__subcmd__source__subcmd__authorise__subcmd__endpoint"
                ;;
            searchright__subcmd__help__subcmd__source,discovery-candidates)
                cmd="searchright__subcmd__help__subcmd__source__subcmd__discovery__subcmd__candidates"
                ;;
            searchright__subcmd__help__subcmd__source,list)
                cmd="searchright__subcmd__help__subcmd__source__subcmd__list"
                ;;
            searchright__subcmd__help__subcmd__source,plan-licensed-request)
                cmd="searchright__subcmd__help__subcmd__source__subcmd__plan__subcmd__licensed__subcmd__request"
                ;;
            searchright__subcmd__help__subcmd__source,validate-discovery-run)
                cmd="searchright__subcmd__help__subcmd__source__subcmd__validate__subcmd__discovery__subcmd__run"
                ;;
            searchright__subcmd__help__subcmd__source,verify-provider-component)
                cmd="searchright__subcmd__help__subcmd__source__subcmd__verify__subcmd__provider__subcmd__component"
                ;;
            searchright__subcmd__help__subcmd__strategy,compile)
                cmd="searchright__subcmd__help__subcmd__strategy__subcmd__compile"
                ;;
            searchright__subcmd__help__subcmd__strategy,validate)
                cmd="searchright__subcmd__help__subcmd__strategy__subcmd__validate"
                ;;
            searchright__subcmd__help__subcmd__strategy,validate-benchmark-report)
                cmd="searchright__subcmd__help__subcmd__strategy__subcmd__validate__subcmd__benchmark__subcmd__report"
                ;;
            searchright__subcmd__help__subcmd__strategy,validate-search)
                cmd="searchright__subcmd__help__subcmd__strategy__subcmd__validate__subcmd__search"
                ;;
            searchright__subcmd__help__subcmd__strategy,validate-standard-assessment)
                cmd="searchright__subcmd__help__subcmd__strategy__subcmd__validate__subcmd__standard__subcmd__assessment"
                ;;
            searchright__subcmd__help__subcmd__strategy,validate-standard-pack)
                cmd="searchright__subcmd__help__subcmd__strategy__subcmd__validate__subcmd__standard__subcmd__pack"
                ;;
            searchright__subcmd__import,deduplicate)
                cmd="searchright__subcmd__import__subcmd__deduplicate"
                ;;
            searchright__subcmd__import,export-records)
                cmd="searchright__subcmd__import__subcmd__export__subcmd__records"
                ;;
            searchright__subcmd__import,help)
                cmd="searchright__subcmd__import__subcmd__help"
                ;;
            searchright__subcmd__import,records)
                cmd="searchright__subcmd__import__subcmd__records"
                ;;
            searchright__subcmd__import__subcmd__help,deduplicate)
                cmd="searchright__subcmd__import__subcmd__help__subcmd__deduplicate"
                ;;
            searchright__subcmd__import__subcmd__help,export-records)
                cmd="searchright__subcmd__import__subcmd__help__subcmd__export__subcmd__records"
                ;;
            searchright__subcmd__import__subcmd__help,help)
                cmd="searchright__subcmd__import__subcmd__help__subcmd__help"
                ;;
            searchright__subcmd__import__subcmd__help,records)
                cmd="searchright__subcmd__import__subcmd__help__subcmd__records"
                ;;
            searchright__subcmd__plan,evaluate-governance)
                cmd="searchright__subcmd__plan__subcmd__evaluate__subcmd__governance"
                ;;
            searchright__subcmd__plan,help)
                cmd="searchright__subcmd__plan__subcmd__help"
                ;;
            searchright__subcmd__plan,validate)
                cmd="searchright__subcmd__plan__subcmd__validate"
                ;;
            searchright__subcmd__plan,validate-amendment)
                cmd="searchright__subcmd__plan__subcmd__validate__subcmd__amendment"
                ;;
            searchright__subcmd__plan,workflow)
                cmd="searchright__subcmd__plan__subcmd__workflow"
                ;;
            searchright__subcmd__plan__subcmd__help,evaluate-governance)
                cmd="searchright__subcmd__plan__subcmd__help__subcmd__evaluate__subcmd__governance"
                ;;
            searchright__subcmd__plan__subcmd__help,help)
                cmd="searchright__subcmd__plan__subcmd__help__subcmd__help"
                ;;
            searchright__subcmd__plan__subcmd__help,validate)
                cmd="searchright__subcmd__plan__subcmd__help__subcmd__validate"
                ;;
            searchright__subcmd__plan__subcmd__help,validate-amendment)
                cmd="searchright__subcmd__plan__subcmd__help__subcmd__validate__subcmd__amendment"
                ;;
            searchright__subcmd__plan__subcmd__help,workflow)
                cmd="searchright__subcmd__plan__subcmd__help__subcmd__workflow"
                ;;
            searchright__subcmd__report,help)
                cmd="searchright__subcmd__report__subcmd__help"
                ;;
            searchright__subcmd__report,prisma)
                cmd="searchright__subcmd__report__subcmd__prisma"
                ;;
            searchright__subcmd__report,provenance)
                cmd="searchright__subcmd__report__subcmd__provenance"
                ;;
            searchright__subcmd__report,render-diagnostics)
                cmd="searchright__subcmd__report__subcmd__render__subcmd__diagnostics"
                ;;
            searchright__subcmd__report__subcmd__help,help)
                cmd="searchright__subcmd__report__subcmd__help__subcmd__help"
                ;;
            searchright__subcmd__report__subcmd__help,prisma)
                cmd="searchright__subcmd__report__subcmd__help__subcmd__prisma"
                ;;
            searchright__subcmd__report__subcmd__help,provenance)
                cmd="searchright__subcmd__report__subcmd__help__subcmd__provenance"
                ;;
            searchright__subcmd__report__subcmd__help,render-diagnostics)
                cmd="searchright__subcmd__report__subcmd__help__subcmd__render__subcmd__diagnostics"
                ;;
            searchright__subcmd__run,authorise-endpoint)
                cmd="searchright__subcmd__run__subcmd__authorise__subcmd__endpoint"
                ;;
            searchright__subcmd__run,help)
                cmd="searchright__subcmd__run__subcmd__help"
                ;;
            searchright__subcmd__run,inspect-content)
                cmd="searchright__subcmd__run__subcmd__inspect__subcmd__content"
                ;;
            searchright__subcmd__run,validate-document-evidence)
                cmd="searchright__subcmd__run__subcmd__validate__subcmd__document__subcmd__evidence"
                ;;
            searchright__subcmd__run,verify-audit)
                cmd="searchright__subcmd__run__subcmd__verify__subcmd__audit"
                ;;
            searchright__subcmd__run,verify-workflow-trace)
                cmd="searchright__subcmd__run__subcmd__verify__subcmd__workflow__subcmd__trace"
                ;;
            searchright__subcmd__run__subcmd__help,authorise-endpoint)
                cmd="searchright__subcmd__run__subcmd__help__subcmd__authorise__subcmd__endpoint"
                ;;
            searchright__subcmd__run__subcmd__help,help)
                cmd="searchright__subcmd__run__subcmd__help__subcmd__help"
                ;;
            searchright__subcmd__run__subcmd__help,inspect-content)
                cmd="searchright__subcmd__run__subcmd__help__subcmd__inspect__subcmd__content"
                ;;
            searchright__subcmd__run__subcmd__help,validate-document-evidence)
                cmd="searchright__subcmd__run__subcmd__help__subcmd__validate__subcmd__document__subcmd__evidence"
                ;;
            searchright__subcmd__run__subcmd__help,verify-audit)
                cmd="searchright__subcmd__run__subcmd__help__subcmd__verify__subcmd__audit"
                ;;
            searchright__subcmd__run__subcmd__help,verify-workflow-trace)
                cmd="searchright__subcmd__run__subcmd__help__subcmd__verify__subcmd__workflow__subcmd__trace"
                ;;
            searchright__subcmd__screen,help)
                cmd="searchright__subcmd__screen__subcmd__help"
                ;;
            searchright__subcmd__screen,living-diff)
                cmd="searchright__subcmd__screen__subcmd__living__subcmd__diff"
                ;;
            searchright__subcmd__screen,rank)
                cmd="searchright__subcmd__screen__subcmd__rank"
                ;;
            searchright__subcmd__screen,study-graph)
                cmd="searchright__subcmd__screen__subcmd__study__subcmd__graph"
                ;;
            searchright__subcmd__screen,validate-living-lineage)
                cmd="searchright__subcmd__screen__subcmd__validate__subcmd__living__subcmd__lineage"
                ;;
            searchright__subcmd__screen,validate-ranking-calibration)
                cmd="searchright__subcmd__screen__subcmd__validate__subcmd__ranking__subcmd__calibration"
                ;;
            searchright__subcmd__screen__subcmd__help,help)
                cmd="searchright__subcmd__screen__subcmd__help__subcmd__help"
                ;;
            searchright__subcmd__screen__subcmd__help,living-diff)
                cmd="searchright__subcmd__screen__subcmd__help__subcmd__living__subcmd__diff"
                ;;
            searchright__subcmd__screen__subcmd__help,rank)
                cmd="searchright__subcmd__screen__subcmd__help__subcmd__rank"
                ;;
            searchright__subcmd__screen__subcmd__help,study-graph)
                cmd="searchright__subcmd__screen__subcmd__help__subcmd__study__subcmd__graph"
                ;;
            searchright__subcmd__screen__subcmd__help,validate-living-lineage)
                cmd="searchright__subcmd__screen__subcmd__help__subcmd__validate__subcmd__living__subcmd__lineage"
                ;;
            searchright__subcmd__screen__subcmd__help,validate-ranking-calibration)
                cmd="searchright__subcmd__screen__subcmd__help__subcmd__validate__subcmd__ranking__subcmd__calibration"
                ;;
            searchright__subcmd__source,authorise-endpoint)
                cmd="searchright__subcmd__source__subcmd__authorise__subcmd__endpoint"
                ;;
            searchright__subcmd__source,discovery-candidates)
                cmd="searchright__subcmd__source__subcmd__discovery__subcmd__candidates"
                ;;
            searchright__subcmd__source,help)
                cmd="searchright__subcmd__source__subcmd__help"
                ;;
            searchright__subcmd__source,list)
                cmd="searchright__subcmd__source__subcmd__list"
                ;;
            searchright__subcmd__source,plan-licensed-request)
                cmd="searchright__subcmd__source__subcmd__plan__subcmd__licensed__subcmd__request"
                ;;
            searchright__subcmd__source,validate-discovery-run)
                cmd="searchright__subcmd__source__subcmd__validate__subcmd__discovery__subcmd__run"
                ;;
            searchright__subcmd__source,verify-provider-component)
                cmd="searchright__subcmd__source__subcmd__verify__subcmd__provider__subcmd__component"
                ;;
            searchright__subcmd__source__subcmd__help,authorise-endpoint)
                cmd="searchright__subcmd__source__subcmd__help__subcmd__authorise__subcmd__endpoint"
                ;;
            searchright__subcmd__source__subcmd__help,discovery-candidates)
                cmd="searchright__subcmd__source__subcmd__help__subcmd__discovery__subcmd__candidates"
                ;;
            searchright__subcmd__source__subcmd__help,help)
                cmd="searchright__subcmd__source__subcmd__help__subcmd__help"
                ;;
            searchright__subcmd__source__subcmd__help,list)
                cmd="searchright__subcmd__source__subcmd__help__subcmd__list"
                ;;
            searchright__subcmd__source__subcmd__help,plan-licensed-request)
                cmd="searchright__subcmd__source__subcmd__help__subcmd__plan__subcmd__licensed__subcmd__request"
                ;;
            searchright__subcmd__source__subcmd__help,validate-discovery-run)
                cmd="searchright__subcmd__source__subcmd__help__subcmd__validate__subcmd__discovery__subcmd__run"
                ;;
            searchright__subcmd__source__subcmd__help,verify-provider-component)
                cmd="searchright__subcmd__source__subcmd__help__subcmd__verify__subcmd__provider__subcmd__component"
                ;;
            searchright__subcmd__strategy,compile)
                cmd="searchright__subcmd__strategy__subcmd__compile"
                ;;
            searchright__subcmd__strategy,help)
                cmd="searchright__subcmd__strategy__subcmd__help"
                ;;
            searchright__subcmd__strategy,validate)
                cmd="searchright__subcmd__strategy__subcmd__validate"
                ;;
            searchright__subcmd__strategy,validate-benchmark-report)
                cmd="searchright__subcmd__strategy__subcmd__validate__subcmd__benchmark__subcmd__report"
                ;;
            searchright__subcmd__strategy,validate-search)
                cmd="searchright__subcmd__strategy__subcmd__validate__subcmd__search"
                ;;
            searchright__subcmd__strategy,validate-standard-assessment)
                cmd="searchright__subcmd__strategy__subcmd__validate__subcmd__standard__subcmd__assessment"
                ;;
            searchright__subcmd__strategy,validate-standard-pack)
                cmd="searchright__subcmd__strategy__subcmd__validate__subcmd__standard__subcmd__pack"
                ;;
            searchright__subcmd__strategy__subcmd__help,compile)
                cmd="searchright__subcmd__strategy__subcmd__help__subcmd__compile"
                ;;
            searchright__subcmd__strategy__subcmd__help,help)
                cmd="searchright__subcmd__strategy__subcmd__help__subcmd__help"
                ;;
            searchright__subcmd__strategy__subcmd__help,validate)
                cmd="searchright__subcmd__strategy__subcmd__help__subcmd__validate"
                ;;
            searchright__subcmd__strategy__subcmd__help,validate-benchmark-report)
                cmd="searchright__subcmd__strategy__subcmd__help__subcmd__validate__subcmd__benchmark__subcmd__report"
                ;;
            searchright__subcmd__strategy__subcmd__help,validate-search)
                cmd="searchright__subcmd__strategy__subcmd__help__subcmd__validate__subcmd__search"
                ;;
            searchright__subcmd__strategy__subcmd__help,validate-standard-assessment)
                cmd="searchright__subcmd__strategy__subcmd__help__subcmd__validate__subcmd__standard__subcmd__assessment"
                ;;
            searchright__subcmd__strategy__subcmd__help,validate-standard-pack)
                cmd="searchright__subcmd__strategy__subcmd__help__subcmd__validate__subcmd__standard__subcmd__pack"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        searchright)
            opts="-h -V --help --version init plan source strategy run import screen report completions manpage validate-plan validate-strategy validate-document-evidence compile deduplicate prisma verify-audit import-records export-records study-graph validate-search living-diff validate-living-lineage provenance rank inspect-content render-diagnostics evaluate-governance authorise-endpoint validate-amendment validate-standard-pack validate-standard-assessment validate-ranking-calibration validate-discovery-run verify-workflow-trace discovery-candidates verify-provider-component plan-licensed-request validate-benchmark-report providers workflow help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__authorise__subcmd__endpoint)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__compile)
            opts="-h --dialect --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --dialect)
                    COMPREPLY=($(compgen -W "pubmed ovid-medline embase europe-pmc cinahl-ebsco psycinfo-ovid scopus web-of-science crossref openalex clinicaltrials-gov generic-boolean" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__completions)
            opts="-h --help bash elvish fish powershell zsh"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__deduplicate)
            opts="-h --title-threshold --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --title-threshold)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__discovery__subcmd__candidates)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__evaluate__subcmd__governance)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__export__subcmd__records)
            opts="-h --review-id --input-format --output-format --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --review-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --input-format)
                    COMPREPLY=($(compgen -W "searchright-json json-lines csl-json ris nbib csv" -- "${cur}"))
                    return 0
                    ;;
                --output-format)
                    COMPREPLY=($(compgen -W "searchright-json json-lines csl-json ris nbib csv" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help)
            opts="init plan source strategy run import screen report completions manpage validate-plan validate-strategy validate-document-evidence compile deduplicate prisma verify-audit import-records export-records study-graph validate-search living-diff validate-living-lineage provenance rank inspect-content render-diagnostics evaluate-governance authorise-endpoint validate-amendment validate-standard-pack validate-standard-assessment validate-ranking-calibration validate-discovery-run verify-workflow-trace discovery-candidates verify-provider-component plan-licensed-request validate-benchmark-report providers workflow help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__authorise__subcmd__endpoint)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__compile)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__completions)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__deduplicate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__discovery__subcmd__candidates)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__evaluate__subcmd__governance)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__export__subcmd__records)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__import)
            opts="records export-records deduplicate"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__import__subcmd__records)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__import__subcmd__deduplicate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__import__subcmd__export__subcmd__records)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__import__subcmd__records)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__init)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__inspect__subcmd__content)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__living__subcmd__diff)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__manpage)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__plan)
            opts="validate validate-amendment evaluate-governance workflow"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__plan__subcmd__licensed__subcmd__request)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__plan__subcmd__evaluate__subcmd__governance)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__plan__subcmd__validate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__plan__subcmd__validate__subcmd__amendment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__plan__subcmd__workflow)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__prisma)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__provenance)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__providers)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__rank)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__render__subcmd__diagnostics)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__report)
            opts="prisma provenance render-diagnostics"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__report__subcmd__prisma)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__report__subcmd__provenance)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__report__subcmd__render__subcmd__diagnostics)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__run)
            opts="authorise-endpoint verify-audit verify-workflow-trace inspect-content validate-document-evidence"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__run__subcmd__authorise__subcmd__endpoint)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__run__subcmd__inspect__subcmd__content)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__run__subcmd__validate__subcmd__document__subcmd__evidence)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__run__subcmd__verify__subcmd__audit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__run__subcmd__verify__subcmd__workflow__subcmd__trace)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__screen)
            opts="rank study-graph validate-ranking-calibration living-diff validate-living-lineage"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__screen__subcmd__living__subcmd__diff)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__screen__subcmd__rank)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__screen__subcmd__study__subcmd__graph)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__screen__subcmd__validate__subcmd__living__subcmd__lineage)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__screen__subcmd__validate__subcmd__ranking__subcmd__calibration)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__source)
            opts="list authorise-endpoint validate-discovery-run discovery-candidates verify-provider-component plan-licensed-request"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__source__subcmd__authorise__subcmd__endpoint)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__source__subcmd__discovery__subcmd__candidates)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__source__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__source__subcmd__plan__subcmd__licensed__subcmd__request)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__source__subcmd__validate__subcmd__discovery__subcmd__run)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__source__subcmd__verify__subcmd__provider__subcmd__component)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__strategy)
            opts="validate compile validate-search validate-standard-pack validate-standard-assessment validate-benchmark-report"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__strategy__subcmd__compile)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__strategy__subcmd__validate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__strategy__subcmd__validate__subcmd__benchmark__subcmd__report)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__strategy__subcmd__validate__subcmd__search)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__strategy__subcmd__validate__subcmd__standard__subcmd__assessment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__strategy__subcmd__validate__subcmd__standard__subcmd__pack)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__study__subcmd__graph)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__validate__subcmd__amendment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__validate__subcmd__benchmark__subcmd__report)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__validate__subcmd__discovery__subcmd__run)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__validate__subcmd__document__subcmd__evidence)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__validate__subcmd__living__subcmd__lineage)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__validate__subcmd__plan)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__validate__subcmd__ranking__subcmd__calibration)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__validate__subcmd__search)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__validate__subcmd__standard__subcmd__assessment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__validate__subcmd__standard__subcmd__pack)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__validate__subcmd__strategy)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__verify__subcmd__audit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__verify__subcmd__provider__subcmd__component)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__verify__subcmd__workflow__subcmd__trace)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__help__subcmd__workflow)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__import)
            opts="-h --help records export-records deduplicate help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__import__subcmd__records)
            opts="-h --format --source-receipt-id --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --format)
                    COMPREPLY=($(compgen -W "searchright-json json-lines csl-json ris nbib csv" -- "${cur}"))
                    return 0
                    ;;
                --source-receipt-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__import__subcmd__deduplicate)
            opts="-h --title-threshold --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --title-threshold)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__import__subcmd__export__subcmd__records)
            opts="-h --review-id --input-format --output-format --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --review-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --input-format)
                    COMPREPLY=($(compgen -W "searchright-json json-lines csl-json ris nbib csv" -- "${cur}"))
                    return 0
                    ;;
                --output-format)
                    COMPREPLY=($(compgen -W "searchright-json json-lines csl-json ris nbib csv" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__import__subcmd__help)
            opts="records export-records deduplicate help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__import__subcmd__help__subcmd__deduplicate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__import__subcmd__help__subcmd__export__subcmd__records)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__import__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__import__subcmd__help__subcmd__records)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__import__subcmd__records)
            opts="-h --format --source-receipt-id --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --format)
                    COMPREPLY=($(compgen -W "searchright-json json-lines csl-json ris nbib csv" -- "${cur}"))
                    return 0
                    ;;
                --source-receipt-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__init)
            opts="-h --target --apply --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --target)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__inspect__subcmd__content)
            opts="-h --subject-id --policy --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --subject-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --policy)
                    COMPREPLY=($(compgen -W "data-only sanitise-then-data-only human-inspection-required" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__living__subcmd__diff)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__manpage)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__plan)
            opts="-h --help validate validate-amendment evaluate-governance workflow help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__plan__subcmd__licensed__subcmd__request)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__plan__subcmd__evaluate__subcmd__governance)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__plan__subcmd__help)
            opts="validate validate-amendment evaluate-governance workflow help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__plan__subcmd__help__subcmd__evaluate__subcmd__governance)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__plan__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__plan__subcmd__help__subcmd__validate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__plan__subcmd__help__subcmd__validate__subcmd__amendment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__plan__subcmd__help__subcmd__workflow)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__plan__subcmd__validate)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__plan__subcmd__validate__subcmd__amendment)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__plan__subcmd__workflow)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__prisma)
            opts="-h --format --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --format)
                    COMPREPLY=($(compgen -W "json mermaid ledger" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__provenance)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__providers)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__rank)
            opts="-h --query-term --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --query-term)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__render__subcmd__diagnostics)
            opts="-h --format --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --format)
                    COMPREPLY=($(compgen -W "plain-text json json-lines" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__report)
            opts="-h --help prisma provenance render-diagnostics help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__report__subcmd__help)
            opts="prisma provenance render-diagnostics help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__report__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__report__subcmd__help__subcmd__prisma)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__report__subcmd__help__subcmd__provenance)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__report__subcmd__help__subcmd__render__subcmd__diagnostics)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__report__subcmd__prisma)
            opts="-h --format --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --format)
                    COMPREPLY=($(compgen -W "json mermaid ledger" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__report__subcmd__provenance)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__report__subcmd__render__subcmd__diagnostics)
            opts="-h --format --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --format)
                    COMPREPLY=($(compgen -W "plain-text json json-lines" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run)
            opts="-h --help authorise-endpoint verify-audit verify-workflow-trace inspect-content validate-document-evidence help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run__subcmd__authorise__subcmd__endpoint)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run__subcmd__help)
            opts="authorise-endpoint verify-audit verify-workflow-trace inspect-content validate-document-evidence help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run__subcmd__help__subcmd__authorise__subcmd__endpoint)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run__subcmd__help__subcmd__inspect__subcmd__content)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run__subcmd__help__subcmd__validate__subcmd__document__subcmd__evidence)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run__subcmd__help__subcmd__verify__subcmd__audit)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run__subcmd__help__subcmd__verify__subcmd__workflow__subcmd__trace)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run__subcmd__inspect__subcmd__content)
            opts="-h --subject-id --policy --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --subject-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --policy)
                    COMPREPLY=($(compgen -W "data-only sanitise-then-data-only human-inspection-required" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run__subcmd__validate__subcmd__document__subcmd__evidence)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run__subcmd__verify__subcmd__audit)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__run__subcmd__verify__subcmd__workflow__subcmd__trace)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen)
            opts="-h --help rank study-graph validate-ranking-calibration living-diff validate-living-lineage help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen__subcmd__help)
            opts="rank study-graph validate-ranking-calibration living-diff validate-living-lineage help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen__subcmd__help__subcmd__living__subcmd__diff)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen__subcmd__help__subcmd__rank)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen__subcmd__help__subcmd__study__subcmd__graph)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen__subcmd__help__subcmd__validate__subcmd__living__subcmd__lineage)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen__subcmd__help__subcmd__validate__subcmd__ranking__subcmd__calibration)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen__subcmd__living__subcmd__diff)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen__subcmd__rank)
            opts="-h --query-term --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --query-term)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen__subcmd__study__subcmd__graph)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen__subcmd__validate__subcmd__living__subcmd__lineage)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__screen__subcmd__validate__subcmd__ranking__subcmd__calibration)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source)
            opts="-h --help list authorise-endpoint validate-discovery-run discovery-candidates verify-provider-component plan-licensed-request help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__authorise__subcmd__endpoint)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__discovery__subcmd__candidates)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__help)
            opts="list authorise-endpoint validate-discovery-run discovery-candidates verify-provider-component plan-licensed-request help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__help__subcmd__authorise__subcmd__endpoint)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__help__subcmd__discovery__subcmd__candidates)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__help__subcmd__list)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__help__subcmd__plan__subcmd__licensed__subcmd__request)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__help__subcmd__validate__subcmd__discovery__subcmd__run)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__help__subcmd__verify__subcmd__provider__subcmd__component)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__list)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__plan__subcmd__licensed__subcmd__request)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__validate__subcmd__discovery__subcmd__run)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__source__subcmd__verify__subcmd__provider__subcmd__component)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy)
            opts="-h --help validate compile validate-search validate-standard-pack validate-standard-assessment validate-benchmark-report help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__compile)
            opts="-h --dialect --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --dialect)
                    COMPREPLY=($(compgen -W "pubmed ovid-medline embase europe-pmc cinahl-ebsco psycinfo-ovid scopus web-of-science crossref openalex clinicaltrials-gov generic-boolean" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__help)
            opts="validate compile validate-search validate-standard-pack validate-standard-assessment validate-benchmark-report help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__help__subcmd__compile)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__help__subcmd__validate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__help__subcmd__validate__subcmd__benchmark__subcmd__report)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__help__subcmd__validate__subcmd__search)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__help__subcmd__validate__subcmd__standard__subcmd__assessment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__help__subcmd__validate__subcmd__standard__subcmd__pack)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__validate)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__validate__subcmd__benchmark__subcmd__report)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__validate__subcmd__search)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__validate__subcmd__standard__subcmd__assessment)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__strategy__subcmd__validate__subcmd__standard__subcmd__pack)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__study__subcmd__graph)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__validate__subcmd__amendment)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__validate__subcmd__benchmark__subcmd__report)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__validate__subcmd__discovery__subcmd__run)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__validate__subcmd__document__subcmd__evidence)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__validate__subcmd__living__subcmd__lineage)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__validate__subcmd__plan)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__validate__subcmd__ranking__subcmd__calibration)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__validate__subcmd__search)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__validate__subcmd__standard__subcmd__assessment)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__validate__subcmd__standard__subcmd__pack)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__validate__subcmd__strategy)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__verify__subcmd__audit)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__verify__subcmd__provider__subcmd__component)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__verify__subcmd__workflow__subcmd__trace)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        searchright__subcmd__workflow)
            opts="-h --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _searchright -o nosort -o bashdefault -o default searchright
else
    complete -F _searchright -o bashdefault -o default searchright
fi
