/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use std::sync::Arc;

use buck2_core::target::label::label::TargetLabel;
use buck2_node::attrs::coerced_deps_collector::CoercedDeps;
use buck2_node::attrs::coerced_deps_collector::CoercedDepsCollector;
use buck2_node::attrs::inspect_options::AttrInspectOptions;
use buck2_node::call_stack::StarlarkCallStack;
use buck2_node::nodes::unconfigured::TargetNode;
use buck2_node::package::Package;
use buck2_node::rule::Rule;
use dupe::Dupe;
use starlark::eval::CallStack;
use starlark::eval::ParametersParser;

use crate::call_stack::StarlarkCallStackWrapper;
use crate::interpreter::module_internals::ModuleInternals;
use crate::nodes::attr_spec::AttributeSpecExt;

pub trait TargetNodeExt: Sized {
    fn from_params_ignore_attrs_for_profiling<'v>(
        rule: Arc<Rule>,
        package: Arc<Package>,
        internals: &ModuleInternals,
        param_parser: &mut ParametersParser<'v, '_>,
    ) -> anyhow::Result<Self>;

    fn from_params<'v>(
        rule: Arc<Rule>,
        package: Arc<Package>,
        internals: &ModuleInternals,
        param_parser: &mut ParametersParser<'v, '_>,
        arg_count: usize,
        ignore_attrs_for_profiling: bool,
        call_stack: Option<CallStack>,
    ) -> anyhow::Result<Self>;
}

impl TargetNodeExt for TargetNode {
    /// Extract only the name attribute from rule arguments, ignore the others.
    fn from_params_ignore_attrs_for_profiling<'v>(
        rule: Arc<Rule>,
        package: Arc<Package>,
        internals: &ModuleInternals,
        param_parser: &mut ParametersParser<'v, '_>,
    ) -> anyhow::Result<Self> {
        let (name, _indices, attr_values) = rule.attributes.start_parse(param_parser, 1)?;

        let label = TargetLabel::new(internals.buildfile_path().package().dupe(), name);
        Ok(TargetNode::new(
            rule.dupe(),
            package,
            label,
            attr_values,
            CoercedDeps::default(),
            None,
        ))
    }

    /// The body of the callable returned by `rule()`. Records the target in this package's `TargetMap`
    #[allow(clippy::box_collection)] // Parameter `call_stack`, because this is the field type.
    fn from_params<'v>(
        rule: Arc<Rule>,
        package: Arc<Package>,
        internals: &ModuleInternals,
        param_parser: &mut ParametersParser<'v, '_>,
        arg_count: usize,
        ignore_attrs_for_profiling: bool,
        call_stack: Option<CallStack>,
    ) -> anyhow::Result<Self> {
        if ignore_attrs_for_profiling {
            return Self::from_params_ignore_attrs_for_profiling(
                rule,
                package,
                internals,
                param_parser,
            );
        }

        let (target_name, attr_values) =
            rule.attributes
                .parse_params(param_parser, arg_count, internals)?;
        let package_name = internals.buildfile_path().package();

        let label = TargetLabel::new(package_name.dupe(), target_name);
        let mut deps_cache = CoercedDepsCollector::new();

        for a in rule.attributes.attrs(&attr_values, AttrInspectOptions::All) {
            a.traverse(label.pkg(), &mut deps_cache)?;
        }

        Ok(TargetNode::new(
            rule,
            package,
            label,
            attr_values,
            CoercedDeps::from(deps_cache),
            call_stack
                .map(StarlarkCallStackWrapper)
                .map(StarlarkCallStack::new),
        ))
    }
}
