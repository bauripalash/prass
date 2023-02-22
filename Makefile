perf:
	cargo flamegraph
	perf script -F +pid > pras.perf

vmb:
	cargo bench --bench vm_bench
