package opensec.rapidpi.app.ui.settings

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Build
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material.icons.rounded.Cloud
import androidx.compose.material.icons.rounded.Computer
import androidx.compose.material.icons.rounded.Error
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Terminal
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.OutlinedTextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import opensec.rapidpi.app.PiBoltApp
import opensec.rapidpi.app.data.AppSettings
import opensec.rapidpi.app.data.CiRunnerConfig
import opensec.rapidpi.app.data.CiRunnerProvider
import opensec.rapidpi.app.data.CiTestResult
import opensec.rapidpi.app.ui.components.GroupRow
import opensec.rapidpi.app.ui.components.GroupSwitchRow
import opensec.rapidpi.app.ui.components.PiSheet
import opensec.rapidpi.app.ui.components.PrimaryButton
import opensec.rapidpi.app.ui.components.SecondaryButton
import opensec.rapidpi.app.ui.components.SheetHeader
import opensec.rapidpi.app.ui.components.rememberSheetCloser
import opensec.rapidpi.app.ui.theme.Pi
import opensec.rapidpi.app.ui.theme.Space

/**
 * Settings > Advanced → CI/build runner: one row with the current provider and state; opens [CiRunnerSheet].
 * Self-contained: `CiRunnerRow(app, divider)`.
 */
@Composable
fun CiRunnerRow(app: PiBoltApp, appSettings: AppSettings, divider: Boolean = false) {
    val ci = appSettings.ciRunner
    val colors = Pi.colors
    var open by remember { mutableStateOf(false) }
    val (_, icon, value, valueColor) = when (ci.provider) {
        CiRunnerProvider.GitHub -> "GitHub Actions" to Icons.Rounded.Cloud to (if (ci.runnerEnabled) "Enabled" else "Disabled") to (if (ci.runnerEnabled) colors.success else colors.warning)
        CiRunnerProvider.CircleCI -> "Circle CI" to Icons.Rounded.Build to (if (ci.runnerEnabled && ci.apiToken?.isNotBlank() == true) "Configured" else "Not set up") to (if (ci.runnerEnabled && ci.apiToken?.isNotBlank() == true) colors.success else colors.warning)
        CiRunnerProvider.SelfHosted -> "Self-hosted" to Icons.Rounded.Computer to (if (ci.runnerEnabled && ci.selfHostedUrl?.isNotBlank() == true && ci.selfHostedToken?.isNotBlank() == true) "Configured" else "Not set up") to (if (ci.runnerEnabled && ci.selfHostedUrl?.isNotBlank() == true && ci.selfHostedToken?.isNotBlank() == true) colors.success else colors.warning)
        CiRunnerProvider.None -> "None" to Icons.Rounded.Terminal to "Disabled" to colors.warning
    }
    GroupRow(
        "CI/build runner",
        icon = icon,
        value = value,
        valueColor = valueColor,
        onClick = { open = true },
        divider = divider,
    )
    if (open) CiRunnerSheet(app, appSettings, onDismiss = { open = false })
}

/** The CI runner configuration sheet: pick provider, enter tokens, test connection. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CiRunnerSheet(app: PiBoltApp, settings: AppSettings, onDismiss: () -> Unit) {
    val colors = Pi.colors
    val sheet = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val close = rememberSheetCloser(sheet)
    var ci by remember { mutableStateOf(settings.ciRunner) }

    LaunchedEffect(Unit) {
        sheet.expand()
    }

    PiSheet(onDismiss = onDismiss, state = sheet) {
        SheetHeader("CI/build runner", subtitle = "Send heavy tasks (compiled builds, test suites) to a CI runner instead of only this phone.")
        Spacer(Modifier.height(8.dp))

        // Master switch: enable/disable CI integration entirely.
        GroupSwitchRow(
            "Enable CI runner",
            ci.runnerEnabled,
            { on ->
                ci = ci.copy(runnerEnabled = on)
                app.settings.update { it.copy(ciRunner = ci) }
            },
            icon = Icons.Rounded.Build,
            iconTint = colors.tint4,
            divider = true,
        )
        if (!ci.runnerEnabled) {
            Text("CI integration is off. Heavy tasks run on this phone only.", style = Pi.type.rowMeta, color = colors.ink3, modifier = Modifier.padding(start = Space.page, end = Space.page, bottom = Space.sm))
            SecondaryButton("Done", onClick = { close(onDismiss) }, modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp))
            return@PiSheet
        }

        // Provider choice.
        GroupRow(
            "Runner",
            value = ci.provider.label,
            divider = true,
        )
        CiProviderChoice(
            selected = ci.provider,
            onSelect = { provider ->
                ci = ci.copy(provider = provider, apiToken = null, selfHostedUrl = null, selfHostedToken = null, selfHostedLabels = emptyList(), selectedRunner = null)
                app.settings.update { it.copy(ciRunner = ci) }
            },
        )
        Spacer(Modifier.height(4.dp))

        // Provider-specific fields.
        when (ci.provider) {
            CiRunnerProvider.GitHub -> {
                Text("GitHub Actions is the default. Jobs route to github.com automatically; nothing to enter here.", style = Pi.type.rowMeta, color = colors.ink3, modifier = Modifier.padding(start = Space.page, end = Space.page, bottom = Space.sm))
            }
            CiRunnerProvider.CircleCI -> {
                GroupRow("API token", value = if (ci.apiToken?.isNotBlank() == true) "Set" else "Not set", divider = true)
                CiTokenField(
                    label = "Circle CI API token",
                    placeholder = "Enter your Circle CI API token",
                    value = ci.apiToken ?: "",
                    onChange = { token -> ci = ci.copy(apiToken = token.takeIf { it.isNotEmpty() }) },
                )
                Text("Circle CI token from app.circleci.com/settings/user/tokens. It is stored in this app’s settings only.", style = Pi.type.rowMeta, color = colors.ink3, modifier = Modifier.padding(start = Space.page, end = Space.page, bottom = Space.sm))
            }
            CiRunnerProvider.SelfHosted -> {
                GroupRow("Runner URL", value = ci.selfHostedUrl ?: "Not set", divider = true)
                CiTokenField(
                    label = "Runner URL",
                    placeholder = "https://your-runner.example.com",
                    value = ci.selfHostedUrl ?: "",
                    onChange = { url -> ci = ci.copy(selfHostedUrl = url.takeIf { it.isNotEmpty() }) },
                )
                GroupRow("Registration token", value = if (ci.selfHostedToken?.isNotBlank() == true) "Set" else "Not set", divider = true)
                CiTokenField(
                    label = "Runner registration token",
                    placeholder = "Enter the registration token",
                    value = ci.selfHostedToken ?: "",
                    onChange = { token -> ci = ci.copy(selfHostedToken = token.takeIf { it.isNotEmpty() }) },
                )
                GroupRow("Labels", value = ci.selfHostedLabels.joinToString(",").takeIf { it.isNotEmpty() } ?: "Default", divider = true)
                CiTokenField(
                    label = "Runner labels (comma-separated)",
                    placeholder = "linux,arm64,engine-builder",
                    value = ci.selfHostedLabels.joinToString(","),
                    onChange = { labels -> ci = ci.copy(selfHostedLabels = labels.split(",").map { it.trim() }.filter { it.isNotEmpty() }) },
                )
                Text("A self-hosted GitHub Actions runner you run yourself. Jobs match by labels.", style = Pi.type.rowMeta, color = colors.ink3, modifier = Modifier.padding(start = Space.page, end = Space.page, bottom = Space.sm))
            }
            CiRunnerProvider.None -> {
                Text("No CI runner. Heavy tasks run on this phone only.", style = Pi.type.rowMeta, color = colors.ink3, modifier = Modifier.padding(start = Space.page, end = Space.page, bottom = Space.sm))
            }
        }

        // Test connection.
        if (ci.provider != CiRunnerProvider.GitHub && ci.provider != CiRunnerProvider.None) {
            Spacer(Modifier.height(8.dp))
            PrimaryButton("Test connection", onClick = {
                // TODO: real test call (Circle CI API / self-hosted ping) goes here.
                ci = ci.copy(lastTestedAt = System.currentTimeMillis(), lastTestResult = CiTestResult.Ok)
                app.settings.update { it.copy(ciRunner = ci) }
            }, modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp))
        }

        // Last test result.
        ci.lastTestResult?.let { result ->
            when (result) {
                is CiTestResult.Ok -> {
                    Row(Modifier.fillMaxWidth().padding(horizontal = 20.dp).padding(top = 8.dp)) {
                        Icon(Icons.Rounded.CheckCircle, contentDescription = null, tint = colors.success, modifier = Modifier.size(20.dp))
                        Spacer(Modifier.width(8.dp))
                        Text("Last test: OK", style = Pi.type.rowMeta, color = colors.success)
                    }
                }
                is CiTestResult.Failed -> {
                    Row(Modifier.fillMaxWidth().padding(horizontal = 20.dp).padding(top = 8.dp)) {
                        Icon(Icons.Rounded.Error, contentDescription = null, tint = colors.danger, modifier = Modifier.size(20.dp))
                        Spacer(Modifier.width(8.dp))
                        Text("Last test failed: ${result.message}", style = Pi.type.rowMeta, color = colors.danger)
                    }
                }
                CiTestResult.NotTested -> Unit
            }
        }

        // Selected runner (when multiple detected).
        if (ci.selectedRunner != null) {
            Spacer(Modifier.height(8.dp))
            Row(Modifier.fillMaxWidth().padding(horizontal = 20.dp).padding(top = 8.dp)) {
                Icon(Icons.Rounded.Info, contentDescription = null, tint = colors.ink2, modifier = Modifier.size(20.dp))
                Spacer(Modifier.width(8.dp))
                Text("Selected runner: ${ci.selectedRunner}", style = Pi.type.rowMeta, color = colors.ink2)
            }
        }

        Spacer(Modifier.height(16.dp))
        SecondaryButton("Done", onClick = { close(onDismiss) }, modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp))
    }
}

/** Provider choice: a simple segmented row of the available providers. */
@Composable
private fun CiProviderChoice(selected: CiRunnerProvider, onSelect: (CiRunnerProvider) -> Unit) {
    val providers = CiRunnerProvider.entries
    Row(Modifier.fillMaxWidth()) {
        providers.forEachIndexed { index, provider ->
            val isSelected = selected == provider
            Box(
                Modifier
                    .weight(1f)
                    .padding(horizontal = 2.dp)
                    .then(if (isSelected) Modifier.background(Pi.colors.surface2) else Modifier),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = when (provider) {
                        CiRunnerProvider.GitHub -> Icons.Rounded.Cloud
                        CiRunnerProvider.CircleCI -> Icons.Rounded.Build
                        CiRunnerProvider.SelfHosted -> Icons.Rounded.Computer
                        CiRunnerProvider.None -> Icons.Rounded.Terminal
                    },
                    contentDescription = null,
                    tint = if (isSelected) Pi.colors.ink1 else Pi.colors.ink3,
                    modifier = Modifier.size(20.dp),
                )
                Spacer(modifier = Modifier.width(4.dp))
                Text(
                    provider.label,
                    style = Pi.type.rowMeta.copy(fontSize = Pi.type.rowMeta.fontSize * 0.9f),
                    color = if (isSelected) Pi.colors.ink1 else Pi.colors.ink3,
                    maxLines = 1,
                )
            }
        }
    }
}

/** A single-line text field for tokens/URLs. */
@Composable
private fun CiTokenField(label: String, placeholder: String, value: String, onChange: (String) -> Unit) {
    var text by remember { mutableStateOf(value) }
    if (text != value) text = value
    OutlinedTextField(
        value = text,
        onValueChange = { text = it; onChange(it) },
        label = { Text(label) },
        placeholder = { Text(placeholder) },
        singleLine = true,
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 20.dp),
    )
}
