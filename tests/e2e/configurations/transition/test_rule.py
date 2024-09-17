# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

# pyre-strict


import json
import re

from buck2.tests.e2e_util.api.buck import Buck
from buck2.tests.e2e_util.buck_workspace import buck_test


def _replace_hash(s: str) -> str:
    return re.sub(r"\b[0-9a-f]{16}\b", "<HASH>", s)


@buck_test(inplace=True)
async def test_configuration_transition_rule_cquery(buck: Buck) -> None:
    # For the reference, cquery output is: P467297091. Note the "forward" node.
    result = await buck.cquery(
        "deps(fbcode//buck2/tests/targets/configurations/transition/rule:the-test)"
    )
    result.check_returncode()
    # Watchos resource should be present twice: as forward and as transitioned.
    assert result.stdout.count(":watchos-resource") == 2
    # No transition for default resource, so it appears once in cquery output.
    assert result.stdout.count(":default-resource") == 1


@buck_test(inplace=True)
async def test_configuration_transition_rule_cquery_actual_attr(buck: Buck) -> None:
    result = await buck.cquery(
        "--target-platforms=fbcode//buck2/tests/targets/configurations/transition/rule:iphoneos-p",
        "fbcode//buck2/tests/targets/configurations/transition/rule:watchos-resource",
        "--output-attribute=actual",
    )
    result.check_returncode()
    q = json.loads(result.stdout)

    # Each key in the JSON output is a different configuration of the same rule `watchos-resource`
    configuration_default = "fbcode//buck2/tests/targets/configurations/transition/rule:watchos-resource (<transitioned-to-watch>#<HASH>)"
    configuration_transition = "fbcode//buck2/tests/targets/configurations/transition/rule:watchos-resource (fbcode//buck2/tests/targets/configurations/transition/rule:iphoneos-p#<HASH>)"
    configurations = [_replace_hash(c) for c in q.keys()]
    assert configuration_default in configurations
    assert configuration_transition in configurations

    config_default_has_attribute_actual = False
    config_transition_has_no_attributes = False
    for config in q.keys():
        if q[config].get("actual"):
            config_default_has_attribute_actual = True
        if not q[config].values():
            config_transition_has_no_attributes = True

    assert config_default_has_attribute_actual
    assert config_transition_has_no_attributes


@buck_test(inplace=True)
async def test_configuration_transition_rule_build(buck: Buck) -> None:
    # Rule implementations do the assertions.
    result = await buck.build(
        "fbcode//buck2/tests/targets/configurations/transition/rule:the-test"
    )
    result.check_returncode()