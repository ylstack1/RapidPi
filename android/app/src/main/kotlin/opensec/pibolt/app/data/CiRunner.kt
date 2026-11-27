package opensec.rapidpi.app.data

/**
 * The CI/build runner a session can use for heavy tasks (compiled engine builds, test suites, desktop
 * builds). Default is GitHub Actions; Circle CI and self-hosted runners are optional and configured here.
 *
 * [provider] is the backend: github, circleci, selfHosted or none.
 * [apiToken] holds the Circle CI API token (kept in the app's settings, never logged).
 * [selfHostedUrl] / [selfHostedToken] / [selfHostedLabels] configure a self-hosted GitHub Actions runner.
 * [selectedRunner] records the runner the user picked when more than one was detected (asked once).
 * [runnerEnabled] is the master switch agents read to decide whether CI integration is active.
 * [lastTestedAt] / [lastTestResult] cache the last "test connection" outcome so the UI can show it.
 */
data class CiRunnerConfig(
    val provider: CiRunnerProvider = CiRunnerProvider.GitHub,
    val apiToken: String? = null,
    val selfHostedUrl: String? = null,
    val selfHostedToken: String? = null,
    val selfHostedLabels: List<String> = emptyList(),
    val selectedRunner: String? = null,
    val runnerEnabled: Boolean = true,
    val lastTestedAt: Long = 0,
    val lastTestResult: CiTestResult? = null,
) {
    /** True when this phone has a usable runner configured (provider set and enabled). */
    val isConfigured: Boolean
        get() = runnerEnabled && provider != CiRunnerProvider.None && when (provider) {
            CiRunnerProvider.GitHub -> true
            CiRunnerProvider.CircleCI -> apiToken?.isNotBlank() == true
            CiRunnerProvider.SelfHosted -> selfHostedUrl?.isNotBlank() == true && selfHostedToken?.isNotBlank() == true
            CiRunnerProvider.None -> false
        }
}

enum class CiRunnerProvider(val label: String, val setupHint: String) {
    GitHub("GitHub Actions", "Already configured; jobs route to github.com by default"),
    CircleCI("Circle CI", "Enter your Circle CI API token, then test the connection"),
    SelfHosted("Self-hosted", "Enter your runner URL and registration token"),
    None("None", "No CI runner; heavy tasks run only on this phone"),
}

sealed interface CiTestResult {
    data object Ok : CiTestResult
    data class Failed(val message: String) : CiTestResult
    data object NotTested : CiTestResult
}

/**
 * What the agent (Pi) can read to decide whether to dispatch a build/test to CI. Exposed through
 * EngineApi so the coding agent can check at runtime without parsing settings itself.
 */
data class CiRunnerInfo(
    val enabled: Boolean,
    val provider: String,
    val configured: Boolean,
    val selectedRunner: String?,
    val lastTestResult: CiTestResult?,
) {
    val label: String get() = CiRunnerProvider.entries.firstOrNull { it.name == provider }?.label ?: provider
}
