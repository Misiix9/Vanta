<script lang="ts">
	type Field = {
		id: string;
		label: string;
		type: 'text' | 'password' | 'dropdown' | 'checkbox' | 'date';
		placeholder?: string;
		options?: string[];
		required?: boolean;
		defaultValue?: string;
	};

	let {
		fields = [],
		onSubmit,
		submitTitle = 'Submit',
	}: {
		fields: Field[];
		onSubmit?: (values: Record<string, unknown>) => void;
		submitTitle?: string;
	} = $props();

	let values = $state<Record<string, unknown>>({});
	let errors = $state<Record<string, string>>({});

	$effect(() => {
		const initial: Record<string, unknown> = {};
		for (const f of fields) {
			initial[f.id] = f.defaultValue ?? (f.type === 'checkbox' ? 'false' : '');
		}
		values = { ...initial };
	});

	function validate(): boolean {
		const next: Record<string, string> = {};
		for (const f of fields) {
			if (f.required) {
				const v = values[f.id];
				const empty =
					v === undefined ||
					v === null ||
					v === '' ||
					(f.type === 'checkbox' && v === 'false');
				if (empty) {
					next[f.id] = `${f.label} is required`;
				}
			}
		}
		errors = next;
		return Object.keys(next).length === 0;
	}

	function handleSubmit(e: Event) {
		e.preventDefault();
		if (!validate()) return;
		onSubmit?.(values);
	}

	function updateValue(id: string, value: unknown) {
		values = { ...values, [id]: value };
		if (errors[id]) {
			errors = { ...errors, [id]: '' };
		}
	}
</script>

<form class="ext-form v2-stack" onsubmit={handleSubmit}>
	{#each fields as field (field.id)}
		<div class="ext-form-field v2-form-field">
			<label for={field.id} class="ext-form-label v2-form-label">
				{field.label}
				{#if field.required}
					<span class="ext-form-required">*</span>
				{/if}
			</label>

			{#if field.type === 'text' || field.type === 'password'}
				<input
					id={field.id}
					type={field.type}
					placeholder={field.placeholder}
					value={String(values[field.id] ?? '')}
					oninput={(e) => updateValue(field.id, (e.target as HTMLInputElement).value)}
					class="ext-form-input"
					class:error={errors[field.id]}
				/>
			{:else if field.type === 'dropdown'}
				<select
					id={field.id}
					class="ext-form-select"
					class:error={errors[field.id]}
					value={String(values[field.id] ?? '')}
					onchange={(e) => updateValue(field.id, (e.target as HTMLSelectElement).value)}
				>
					<option value="">Select...</option>
					{#each field.options ?? [] as opt}
						<option value={opt}>{opt}</option>
					{/each}
				</select>
			{:else if field.type === 'checkbox'}
				<label class="ext-form-checkbox-wrap">
					<input
						type="checkbox"
						id={field.id}
						checked={values[field.id] === 'true' || values[field.id] === true}
						onchange={(e) =>
							updateValue(field.id, (e.target as HTMLInputElement).checked ? 'true' : 'false')}
						class="ext-form-checkbox"
					/>
					<span class="ext-form-checkbox-label">{field.placeholder ?? 'Yes'}</span>
				</label>
			{:else if field.type === 'date'}
				<input
					id={field.id}
					type="date"
					value={String(values[field.id] ?? '')}
					oninput={(e) => updateValue(field.id, (e.target as HTMLInputElement).value)}
					class="ext-form-input"
					class:error={errors[field.id]}
				/>
			{/if}

			{#if errors[field.id]}
				<span class="ext-form-error">{errors[field.id]}</span>
			{/if}
		</div>
	{/each}

	<button type="submit" class="ext-form-submit">{submitTitle}</button>
</form>

<style>
	.ext-form {
		display: flex;
		flex-direction: column;
		gap: 16px;
		background: var(--vanta-surface, #0c0c0c);
		border-radius: var(--vanta-radius, 12px);
		padding: 16px;
	}

	.ext-form-field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.ext-form-label {
		font-size: 13px;
		font-weight: 600;
		color: var(--vanta-text, #ebebeb);
	}

	.ext-form-required {
		color: var(--vanta-accent, #7b35f0);
	}

	.ext-form-input,
	.ext-form-select {
		width: 100%;
		padding: 10px 12px;
		background: var(--vanta-bg, #000);
		border: 1px solid var(--vanta-border, rgba(123, 53, 240, 0.14));
		border-radius: var(--vanta-radius, 8px);
		color: var(--vanta-text, #ebebeb);
		font-size: 14px;
		outline: none;
		transition: border-color 140ms ease;
	}

	.ext-form-input::placeholder {
		color: var(--vanta-text-dim, #555);
	}

	.ext-form-input:focus,
	.ext-form-select:focus {
		border-color: var(--vanta-accent, #7b35f0);
	}

	.ext-form-input.error,
	.ext-form-select.error {
		border-color: var(--ds-danger);
	}

	.ext-form-checkbox-wrap {
		display: flex;
		align-items: center;
		gap: 10px;
		cursor: pointer;
		color: var(--vanta-text, #ebebeb);
		font-size: 14px;
	}

	.ext-form-checkbox {
		width: 18px;
		height: 18px;
		accent-color: var(--vanta-accent, #7b35f0);
		cursor: pointer;
	}

	.ext-form-checkbox-label {
		color: var(--vanta-text-dim, #888);
	}

	.ext-form-error {
		font-size: 12px;
		color: var(--ds-danger);
	}

	.ext-form-submit {
		margin-top: 8px;
		padding: 12px 20px;
		background: var(--vanta-accent, #7b35f0);
		color: #fff;
		border: none;
		border-radius: var(--vanta-radius, 8px);
		font-weight: 600;
		font-size: 14px;
		cursor: pointer;
		transition: background 140ms ease, box-shadow 140ms ease;
	}

	.ext-form-submit:hover {
		background: var(--vanta-accent-glow, #9b5ff8);
		box-shadow: 0 0 20px var(--vanta-accent-glow, rgba(123, 53, 240, 0.4));
	}
</style>
