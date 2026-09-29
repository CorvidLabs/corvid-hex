---
change: CHG-0006-use-fledge-native-strict-spec-validation-in-hosted-trust-before-the-specsync-act
artifact: context
---

# Context

The hosted Trust action installs Fledge and immediately runs the configured lifecycle lane. Its pinned
SpecSync action runs later, so a lifecycle task that invokes the bare `specsync` executable fails on a
clean runner even though the same task succeeds on a developer machine with SpecSync already installed.

The lifecycle must therefore use Fledge's bundled SpecSync integration. The later Trust contract step
continues to run the immutable SpecSync 5.0.1 action and enforce the repository's committed 100 percent
coverage policy.
