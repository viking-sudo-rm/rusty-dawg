.PHONY : release
release :
	maturin build --release

.PHONY : install
install : release
	pip install --force-reinstall target/wheels/*.whl
