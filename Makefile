# Simple makefile to ease development and testing.

PROJECT:=relox

# =============================================================================

ifdef CI
  Q:=@
  QUIET:=
endif

# =============================================================================

ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

Q?=@
QUIET?=--quiet

SHELL:=/bin/bash

COV_OUT:=$(ROOT_DIR)/target/debug/coverage
GCOV_TOOL:=$(ROOT_DIR)/llvm-gcov-9
LCOV_TOOL:=lcov
GENHTML_TOOL:=genhtml

RUSTC_STABLE_FLAGS:=
RUSTC_NIGHTLY_FLAGS:=" \
  -Ccodegen-units=1 -Copt-level=0 -Cinline-threshold=0 \
  -Clink-dead-code -Coverflow-checks=off -Clink-dead-code \
  -Zprofile -Zno-landing-pads"
RUSTC_CHANNEL:=+stable
RUSTC_FLAGS:=$(RUSTC_STABLE_FLAGS)

EMPTY:=
SPACE:= $(EMPTY) $(EMPTY)
COMMA:= ,

# =============================================================================

TEST_FEATURES:=\
  default host embedded embedded_minimal \
  compress decompress \
  no_std,decompress \
  no_std,no_sanity_check \
  no_std,no_bounds_check

# =============================================================================

.PHONY: build
build:
	$(foreach FEATURES,$(TEST_FEATURES),$(call build_one,$(FEATURES),--release))

.PHONY: test
test:
	$(foreach FEATURES,$(TEST_FEATURES),$(call test_one,$(FEATURES),))

.PHONY: test-coverage
test-coverage: --pre-coverage clean test --post-coverage

.PHONY: open-coverage
open-coverage:
	$(Q)xdg-open $(COV_OUT)/html/index.html

.PHONY: fmt
fmt:
	$(Q)cargo-fmt --verbose

.PHONY: doc
doc:
	$(Q)cargo doc --no-deps --features "compress,decompress"

.PHONY: open-doc
open-doc:
	$(Q)xdg-open $(ROOT_DIR)/target/doc/relox/index.html

.PHONY: clean
clean:
	$(Q)rm -r $(ROOT_DIR)/target 2>/dev/null || exit 0

# =============================================================================

.PHONY: --pre-coverage
--pre-coverage:
	$(call check_dependency,$(LCOV_TOOL),--version)
	$(call check_dependency,$(GCOV_TOOL),--version)
	$(call check_dependency,$(GENHTML_TOOL),--version)
	$(Q)rustup $(QUIET) toolchain install nightly
	$(eval RUSTC_CHANNEL:=+nightly)
	$(eval RUSTC_FLAGS:=$(RUSTC_NIGHTLY_FLAGS))

.PHONY: --post-coverage
--post-coverage:
	$(Q)mkdir -p $(COV_OUT)
	$(Q)$(LCOV_TOOL) $(QUIET) \
		--gcov-tool $(GCOV_TOOL) \
	  --capture \
		--directory target/$(1)/debug \
		--base-directory . \
		--output-file $(COV_OUT)/lcov.all.info \
		--rc lcov_branch_coverage=1 \
		--rc lcov_excl_line=assert \
		2> >(grep -Ev "No such file or directory$$")
	$(Q)$(LCOV_TOOL) $(QUIET) \
		--gcov-tool $(GCOV_TOOL) \
		--extract $(COV_OUT)/lcov.all.info "$(ROOT_DIR)/*" \
		--output-file $(COV_OUT)/lcov.info \
		--rc lcov_branch_coverage=1
	$(Q)$(GENHTML_TOOL) \
		--output-directory $(COV_OUT)/html \
		--title $(PROJECT) --highlight --legend \
		--branch-coverage \
		$(COV_OUT)/lcov.info

# =============================================================================

define check_dependency
  $(Q)$(1) $(2) 2>/dev/null \
    || (echo "$(1) is not found. Please install this required deependency." \
        && exit 1)

endef

define build_one
  cargo build \
    --no-default-features --features $(call feature_flags,$(1)) $(2)

endef

define test_one
  CARGO_INCREMENTAL=0 RUSTFLAGS=$(RUSTC_FLAGS) \
    cargo $(RUSTC_CHANNEL) test \
    --no-default-features --features $(call feature_flags,$(1)) $(2)

endef

define feature_flags
$(subst $(SPACE),$(COMMA),$(1))
endef

# =============================================================================
