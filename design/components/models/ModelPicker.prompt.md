Pick the active model from the .gguf files on disk; goes inside Fieldset "Installed models".
```jsx
<ModelPicker value={model} onChange={setModel} models={[{name:"Qwen3.8-27B-UD-Q4_K_XL.gguf",size:17.6e9},{name:"my-own-model.Q5_K_M.gguf",size:9.1e9}]} />
```
