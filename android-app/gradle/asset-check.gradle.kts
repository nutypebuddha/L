// Apply from app/build.gradle.kts:   apply(from = "../gradle/asset-check.gradle.kts")
//
// Wires the asset parse-check into every assemble. A frontend that cannot parse
// is a build failure, not a runtime surprise.

val checkAssets by tasks.registering(Exec::class) {
    group = "verification"
    description = "Parse-check bundled JS/HTML assets"
    workingDir = rootDir
    commandLine("bash", "check-assets.sh", "app/src/main/assets")
    isIgnoreExitValue = false
}

tasks.matching { it.name.startsWith("merge") && it.name.endsWith("Assets") }
    .configureEach { dependsOn(checkAssets) }
