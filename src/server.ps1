# llama-server command line, built from (validated) settings. Kept free of side effects
# so it can be unit-tested.

# Qwen 3.8's chat template accepts reasoning_effort = low | medium | xhigh ('high' is an
# alias for xhigh) and raises an error for anything else. "No thinking" is a separate
# switch, enable_thinking = false.
function Get-ChatTemplateKwargs([string]$reasoning) {
  if ($reasoning -eq 'none') { '{"enable_thinking":false}' }
  else { '{"reasoning_effort":"' + $reasoning + '"}' }
}

# Returns the argument list for Start-Process (each element already quoted for the
# Windows command line where needed).
function Get-ServerArgs($settings, [string]$modelPath) {
  $kwargs = (Get-ChatTemplateKwargs $settings.Reasoning) -replace '"', '\"'
  $a = @(
    '-m', "`"$modelPath`"",
    '--host', $settings.ListenHost, '--port', $settings.Port,
    '-ngl', '99', '-c', $settings.Context, '-fa', 'on',
    '--cache-type-k', 'q8_0', '--cache-type-v', 'q8_0', '--parallel', '1',
    '--jinja', '--chat-template-kwargs', "`"$kwargs`""
  )
  if ($settings.ApiKey) { $a += '--api-key', $settings.ApiKey }
  # Sampling recommended on the model card: non-thinking vs. thinking mode
  if ($settings.Reasoning -eq 'none') {
    $a += '--temp', '0.7', '--top-p', '0.8', '--top-k', '20', '--min-p', '0', '--presence-penalty', '1.5'
  } else {
    $a += '--temp', '1.0', '--top-p', '0.95', '--top-k', '20', '--min-p', '0'
  }
  $a | ForEach-Object { "$_" }
}
