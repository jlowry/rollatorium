use std::cmp::Ordering;
use std::collections::HashSet;

use rand::RngCore;
use rand::distr::{Distribution, Uniform};

use crate::Result;
use crate::ast::{
    Annotation, BinaryOperator, DiceSize, Node, Selector, SelectorKind, SetOperation, SetOperator,
    UnaryOperator,
};
use crate::error::RollatoriumError::Eval;

const EPSILON: f64 = 1e-9;

#[derive(Debug, Clone)]
pub struct EvalConfig {
    pub max_rolls: usize,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self { max_rolls: 1000 }
    }
}

#[derive(Debug, Clone)]
pub struct EvalResult<T> {
    pub total: f64,
    pub value: Value<T>,
}

#[derive(Debug, Clone)]
pub enum Value<T> {
    Literal(f64),
    Unary {
        operator: UnaryOperator,
        operand: Box<EvalResult<T>>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<EvalResult<T>>,
        right: Box<EvalResult<T>>,
    },
    Dice(DiceRoll<T>),
    Set(SetRoll<T>),
    Annotated {
        expr: Box<EvalResult<T>>,
        annotations: Vec<Annotation<T>>,
    },
}

#[derive(Debug, Clone)]
pub struct DiceRoll<T> {
    pub quantity: usize,
    pub size: u32,
    pub dice: Vec<DieResult>,
    pub operations: Vec<SetOperation<T>>,
}

#[derive(Debug, Clone)]
pub struct DieResult {
    pub value: f64,
    pub rolls: Vec<f64>,
    pub kept: bool,
    pub dropped: bool,
    pub origin: DieOrigin,
    pub adjustments: Vec<DieAdjustment>,
}

impl DieResult {
    fn new(value: f64, origin: DieOrigin) -> Self {
        Self {
            value,
            rolls: vec![value],
            kept: true,
            dropped: false,
            origin,
            adjustments: Vec::new(),
        }
    }

    fn refresh_drop_state(&mut self) {
        self.dropped = !self.kept;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DieOrigin {
    Original,
    RerollAdd,
    Explosion,
}

#[derive(Debug, Clone)]
pub enum DieAdjustment {
    Minimum { threshold: f64, previous: f64 },
    Maximum { threshold: f64, previous: f64 },
}

#[derive(Debug, Clone)]
pub struct SetRoll<T> {
    pub elements: Vec<SetElement<T>>,
    pub operations: Vec<SetOperation<T>>,
}

#[derive(Debug, Clone)]
pub struct SetElement<T> {
    pub value: EvalResult<T>,
    pub kept: bool,
    pub dropped: bool,
}

impl<T> SetElement<T> {
    fn refresh_drop_state(&mut self) {
        self.dropped = !self.kept;
    }

    fn map_tags_inner<U, F>(self, f: &mut F) -> SetElement<U>
    where
        F: FnMut(T) -> U,
    {
        SetElement {
            value: self.value.map_tags_inner(f),
            kept: self.kept,
            dropped: self.dropped,
        }
    }
}

impl<T> EvalResult<T> {
    /// Transform every tag in the evaluated tree, producing an `EvalResult` over
    /// a new tag type. The counterpart to [`crate::Node::map_tags`] for results
    /// that have already been evaluated; only [`Value::Annotated`] wrappers carry
    /// tags, the rest is rebuilt with children mapped.
    pub fn map_tags<U, F>(self, mut f: F) -> EvalResult<U>
    where
        F: FnMut(T) -> U,
    {
        self.map_tags_inner(&mut f)
    }

    fn map_tags_inner<U, F>(self, f: &mut F) -> EvalResult<U>
    where
        F: FnMut(T) -> U,
    {
        EvalResult {
            total: self.total,
            value: self.value.map_tags_inner(f),
        }
    }
}

impl<T> Value<T> {
    fn map_tags_inner<U, F>(self, f: &mut F) -> Value<U>
    where
        F: FnMut(T) -> U,
    {
        match self {
            Value::Literal(v) => Value::Literal(v),
            Value::Unary { operator, operand } => Value::Unary {
                operator,
                operand: Box::new(operand.map_tags_inner(f)),
            },
            Value::Binary {
                operator,
                left,
                right,
            } => Value::Binary {
                operator,
                left: Box::new(left.map_tags_inner(f)),
                right: Box::new(right.map_tags_inner(f)),
            },
            Value::Dice(roll) => Value::Dice(DiceRoll {
                quantity: roll.quantity,
                size: roll.size,
                dice: roll.dice,
                operations: roll
                    .operations
                    .into_iter()
                    .map(|o| o.map_tags_inner(f))
                    .collect(),
            }),
            Value::Set(roll) => Value::Set(SetRoll {
                elements: roll
                    .elements
                    .into_iter()
                    .map(|e| e.map_tags_inner(f))
                    .collect(),
                operations: roll
                    .operations
                    .into_iter()
                    .map(|o| o.map_tags_inner(f))
                    .collect(),
            }),
            Value::Annotated { expr, annotations } => Value::Annotated {
                expr: Box::new(expr.map_tags_inner(f)),
                annotations: annotations
                    .into_iter()
                    .map(|a| Annotation { tag: f(a.tag) })
                    .collect(),
            },
        }
    }
}

pub fn evaluate<T: Clone>(expr: &Node<T>) -> Result<EvalResult<T>> {
    evaluate_with_config(expr, EvalConfig::default())
}

pub fn evaluate_with_config<T: Clone>(
    expr: &Node<T>,
    config: EvalConfig,
) -> Result<EvalResult<T>> {
    evaluate_with_rng(expr, config, rand::rng())
}

pub fn evaluate_with_rng<T, R>(
    expr: &Node<T>,
    config: EvalConfig,
    rng: R,
) -> Result<EvalResult<T>>
where
    T: Clone,
    R: RngCore,
{
    Evaluator {
        rng,
        config,
        rolls: 0,
    }
    .eval(expr)
}

struct Evaluator<R: RngCore> {
    rng: R,
    config: EvalConfig,
    rolls: usize,
}

impl<R: RngCore> Evaluator<R> {
    fn eval<T: Clone>(&mut self, node: &Node<T>) -> Result<EvalResult<T>> {
        match node {
            Node::Literal(v) => Ok(EvalResult {
                total: *v,
                value: Value::Literal(*v),
            }),
            Node::Unary { operator, operand } => {
                let evaluated = self.eval(operand)?;
                let total = match operator {
                    UnaryOperator::Plus => evaluated.total,
                    UnaryOperator::Minus => -evaluated.total,
                };
                Ok(EvalResult {
                    total,
                    value: Value::Unary {
                        operator: *operator,
                        operand: Box::new(evaluated),
                    },
                })
            }
            Node::Binary {
                operator,
                left,
                right,
            } => {
                let left_eval = self.eval(left)?;
                let right_eval = self.eval(right)?;
                let total = match operator {
                    BinaryOperator::Add => left_eval.total + right_eval.total,
                    BinaryOperator::Subtract => left_eval.total - right_eval.total,
                    BinaryOperator::Multiply => left_eval.total * right_eval.total,
                    BinaryOperator::Divide => left_eval.total / right_eval.total,
                    BinaryOperator::IntDivide => (left_eval.total / right_eval.total).trunc(),
                    BinaryOperator::Modulo => left_eval.total % right_eval.total,
                    BinaryOperator::Equal => (left_eval.total == right_eval.total) as i32 as f64,
                    BinaryOperator::NotEqual => (left_eval.total != right_eval.total) as i32 as f64,
                    BinaryOperator::Greater => (left_eval.total > right_eval.total) as i32 as f64,
                    BinaryOperator::GreaterEqual => {
                        (left_eval.total >= right_eval.total) as i32 as f64
                    }
                    BinaryOperator::Less => (left_eval.total < right_eval.total) as i32 as f64,
                    BinaryOperator::LessEqual => {
                        (left_eval.total <= right_eval.total) as i32 as f64
                    }
                };
                Ok(EvalResult {
                    total,
                    value: Value::Binary {
                        operator: *operator,
                        left: Box::new(left_eval),
                        right: Box::new(right_eval),
                    },
                })
            }
            Node::Dice { num, size } => self.eval_dice(num.as_deref(), size, &[]),
            Node::DiceWithOps { dice, operations } => match dice.as_ref() {
                Node::Dice { num, size } => self.eval_dice(num.as_deref(), size, operations),
                _ => Err(Eval("DiceWithOps must contain a dice node".into())),
            },
            Node::Set {
                elements,
                operations,
            } => self.eval_set(elements, operations),
            Node::Annotated { expr, annotations } => {
                let evaluated = self.eval(expr)?;
                Ok(EvalResult {
                    total: evaluated.total,
                    value: Value::Annotated {
                        expr: Box::new(evaluated),
                        annotations: annotations.clone(),
                    },
                })
            }
        }
    }

    fn eval_dice<T: Clone>(
        &mut self,
        quantity: Option<&Node<T>>,
        size: &DiceSize<T>,
        operations: &[SetOperation<T>],
    ) -> Result<EvalResult<T>> {
        let quantity_value = match quantity {
            Some(node) => {
                let result = self.eval(node)?;
                self.as_usize(result.total, "dice quantity")?
            }
            None => 1,
        };

        let (die_low, die_high) = match size {
            DiceSize::Percent => (0u32, 9),
            DiceSize::Value(inner) => {
                let result = self.eval(inner)?;
                (1, self.as_u32(result.total, "die size")?)
            }
        };

        if die_high == 0 {
            return Err(Eval("Die size must be positive".into()));
        }

        let distribution = Uniform::new_inclusive(die_low, die_high)
            .map_err(|err| Eval(format!("Invalid die size {}: {}", die_high, err)))?;
        let mut dice = Vec::with_capacity(quantity_value);
        for _ in 0..quantity_value {
            let roll = self.roll_die(&distribution, size)?;
            dice.push(DieResult::new(roll, DieOrigin::Original));
        }

        self.apply_dice_operations(&mut dice, &distribution, operations, size)?;
        for die in &mut dice {
            die.refresh_drop_state();
        }
        let total: f64 = dice.iter().filter(|d| d.kept).map(|d| d.value).sum();
        Ok(EvalResult {
            total,
            value: Value::Dice(DiceRoll {
                quantity: quantity_value,
                size: die_high,
                dice,
                operations: operations.to_vec(),
            }),
        })
    }

    fn eval_set<T: Clone>(
        &mut self,
        elements: &[Node<T>],
        operations: &[SetOperation<T>],
    ) -> Result<EvalResult<T>> {
        let mut evaluated_elements = Vec::with_capacity(elements.len());
        for element in elements {
            let value = self.eval(element)?;
            evaluated_elements.push(SetElement {
                value,
                kept: true,
                dropped: false,
            });
        }

        self.apply_set_operations(&mut evaluated_elements, operations)?;
        for element in &mut evaluated_elements {
            element.refresh_drop_state();
        }
        let total: f64 = evaluated_elements
            .iter()
            .filter(|e| e.kept)
            .map(|e| e.value.total)
            .sum();
        Ok(EvalResult {
            total,
            value: Value::Set(SetRoll {
                elements: evaluated_elements,
                operations: operations.to_vec(),
            }),
        })
    }

    fn roll_die<T>(&mut self, distribution: &Uniform<u32>, die_size: &DiceSize<T>) -> Result<f64> {
        if self.rolls >= self.config.max_rolls {
            return Err(Eval("Exceeded maximum number of rolls".into()));
        }
        self.rolls += 1;
        let mut value = distribution.sample(&mut self.rng) as f64;
        if matches!(die_size, DiceSize::Percent) {
            value *= 10.0;
        }

        Ok(value)
    }

    fn as_usize(&self, value: f64, context: &str) -> Result<usize> {
        if value < 0.0 {
            return Err(Eval(format!("{} must be non-negative", context)));
        }
        if (value.round() - value).abs() > EPSILON {
            return Err(Eval(format!(
                "{} must be an integer, found {}",
                context, value
            )));
        }
        Ok(value.round() as usize)
    }

    fn as_u32(&self, value: f64, context: &str) -> Result<u32> {
        if value <= 0.0 {
            return Err(Eval(format!("{} must be positive", context)));
        }
        if (value.round() - value).abs() > EPSILON {
            return Err(Eval(format!(
                "{} must be an integer, found {}",
                context, value
            )));
        }
        Ok(value.round() as u32)
    }

    fn apply_dice_operations<T: Clone>(
        &mut self,
        dice: &mut Vec<DieResult>,
        distribution: &Uniform<u32>,
        operations: &[SetOperation<T>],
        size: &DiceSize<T>,
    ) -> Result<()> {
        for operation in operations {
            match operation.operator {
                SetOperator::Keep => {
                    let selected = self.select_dice(dice, &operation.selectors)?;
                    let selected: HashSet<_> = selected.into_iter().collect();
                    for (idx, die) in dice.iter_mut().enumerate() {
                        if die.kept {
                            die.kept = selected.contains(&idx);
                        }
                    }
                }
                SetOperator::Drop => {
                    let selected = self.select_dice(dice, &operation.selectors)?;
                    for idx in selected {
                        if let Some(die) = dice.get_mut(idx) {
                            die.kept = false;
                        }
                    }
                }
                SetOperator::Reroll => loop {
                    let selected = self.select_dice(dice, &operation.selectors)?;
                    if selected.is_empty() {
                        break;
                    }
                    let mut changed = false;
                    for idx in selected {
                        if let Some(die) = dice.get_mut(idx) {
                            let new_value = self.roll_die(distribution, size)?;
                            die.rolls.push(new_value);
                            die.value = new_value;
                            changed = true;
                        }
                    }
                    if !changed {
                        break;
                    }
                },
                SetOperator::RerollOnce => {
                    let selected = self.select_dice(dice, &operation.selectors)?;
                    for idx in selected {
                        if let Some(die) = dice.get_mut(idx) {
                            let new_value = self.roll_die(distribution, size)?;
                            die.rolls.push(new_value);
                            die.value = new_value;
                        }
                    }
                }
                SetOperator::RerollAdd => {
                    let selected = self.select_dice(dice, &operation.selectors)?;
                    for _ in 0..selected.len() {
                        let new_value = self.roll_die(distribution, size)?;
                        dice.push(DieResult::new(new_value, DieOrigin::RerollAdd));
                    }
                }
                SetOperator::Explode => {
                    let mut queue = self.select_dice(dice, &operation.selectors)?;
                    let mut idx = 0;
                    while idx < queue.len() {
                        idx += 1;
                        let new_value = self.roll_die(distribution, size)?;
                        dice.push(DieResult::new(new_value, DieOrigin::Explosion));
                        let new_idx = dice.len() - 1;
                        let matches = self
                            .select_dice(dice, &operation.selectors)?
                            .into_iter()
                            .any(|i| i == new_idx);
                        if matches {
                            queue.push(new_idx);
                        }
                    }
                }
                SetOperator::Minimum => {
                    if operation.selectors.is_empty() {
                        return Err(Eval("Minimum operation requires a selector".into()));
                    }
                    if operation.selectors[0].kind != SelectorKind::Literal {
                        return Err(Eval("selector target must be positive".into()));
                    }
                    let threshold = self.eval(&operation.selectors[0].target)?.total;
                    if threshold <= 0.0 {
                        return Err(Eval("selector target must be positive".into()));
                    }
                    let affected = if operation.selectors.len() > 1 {
                        self.select_dice(dice, &operation.selectors[1..])?
                    } else {
                        dice.iter()
                            .enumerate()
                            .filter(|(_, die)| die.kept)
                            .map(|(idx, _)| idx)
                            .collect()
                    };
                    for idx in affected {
                        if let Some(die) = dice.get_mut(idx)
                            && die.value < threshold
                        {
                            let previous = die.value;
                            die.value = threshold;
                            die.adjustments.push(DieAdjustment::Minimum {
                                threshold,
                                previous,
                            });
                        }
                    }
                }
                SetOperator::Maximum => {
                    if operation.selectors.is_empty() {
                        return Err(Eval("Maximum operation requires a selector".into()));
                    }
                    if operation.selectors[0].kind != SelectorKind::Literal {
                        return Err(Eval("selector target must be positive".into()));
                    }
                    let threshold = self.eval(&operation.selectors[0].target)?.total;
                    let affected = if operation.selectors.len() > 1 {
                        self.select_dice(dice, &operation.selectors[1..])?
                    } else {
                        dice.iter()
                            .enumerate()
                            .filter(|(_, die)| die.kept)
                            .map(|(idx, _)| idx)
                            .collect()
                    };
                    for idx in affected {
                        if let Some(die) = dice.get_mut(idx)
                            && die.value > threshold
                        {
                            let previous = die.value;
                            die.value = threshold;
                            die.adjustments.push(DieAdjustment::Maximum {
                                threshold,
                                previous,
                            });
                        }
                    }
                }
                other => {
                    return Err(Eval(format!(
                        "Set operation {:?} is not supported in the evaluator",
                        other
                    )));
                }
            }
        }
        Ok(())
    }

    fn apply_set_operations<T: Clone>(
        &mut self,
        elements: &mut [SetElement<T>],
        operations: &[SetOperation<T>],
    ) -> Result<()> {
        let mut keep_initialized = false;
        for operation in operations {
            match operation.operator {
                SetOperator::Keep => {
                    let selected =
                        self.select_set_elements(elements, &operation.selectors, false)?;
                    if !keep_initialized {
                        for element in elements.iter_mut() {
                            element.kept = false;
                        }
                        keep_initialized = true;
                    }
                    for idx in selected {
                        if let Some(element) = elements.get_mut(idx) {
                            element.kept = true;
                        }
                    }
                }
                SetOperator::Drop => {
                    let selected =
                        self.select_set_elements(elements, &operation.selectors, true)?;
                    for idx in selected {
                        if let Some(element) = elements.get_mut(idx) {
                            element.kept = false;
                        }
                    }
                }
                other => {
                    return Err(Eval(format!(
                        "Set operation {:?} is not supported for sets",
                        other
                    )));
                }
            }
        }
        Ok(())
    }

    fn select_dice<T: Clone>(
        &mut self,
        dice: &[DieResult],
        selectors: &[Selector<T>],
    ) -> Result<Vec<usize>> {
        if selectors.is_empty() {
            return Ok(Vec::new());
        }
        let mut selected = HashSet::new();
        for selector in selectors {
            let mut indices = match selector.kind {
                SelectorKind::Highest => {
                    let value = self.eval(&selector.target)?.total;
                    let count = self.as_usize(value, "selector")?;
                    self.select_highest(dice, count)
                }
                SelectorKind::Lowest => {
                    let value = self.eval(&selector.target)?.total;
                    let count = self.as_usize(value, "selector")?;
                    self.select_lowest(dice, count)
                }
                SelectorKind::GreaterThan => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_value(dice, |die_value| die_value > value)
                }
                SelectorKind::GreaterThanOrEqual => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_value(dice, |die_value| die_value >= value)
                }
                SelectorKind::LessThan => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_value(dice, |die_value| die_value < value)
                }
                SelectorKind::LessThanOrEqual => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_value(dice, |die_value| die_value <= value)
                }
                SelectorKind::EqualTo => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_value(dice, |die_value| (die_value - value).abs() <= EPSILON)
                }
                SelectorKind::NotEqual => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_value(dice, |die_value| (die_value - value).abs() > EPSILON)
                }
                SelectorKind::Literal => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_value(dice, |die_value| (die_value - value).abs() <= EPSILON)
                }
            }?;
            selected.extend(indices.drain(..));
        }
        let mut collected: Vec<_> = selected.into_iter().collect();
        collected.sort_unstable();
        Ok(collected)
    }

    fn select_set_elements<T: Clone>(
        &mut self,
        elements: &[SetElement<T>],
        selectors: &[Selector<T>],
        only_kept: bool,
    ) -> Result<Vec<usize>> {
        if selectors.is_empty() {
            return Ok(Vec::new());
        }
        let mut selected = HashSet::new();
        for selector in selectors {
            let mut indices = match selector.kind {
                SelectorKind::Highest => {
                    let value = self.eval(&selector.target)?.total;
                    let count = self.as_usize(value, "selector")?;
                    self.select_set_highest(elements, count, only_kept)
                }
                SelectorKind::Lowest => {
                    let value = self.eval(&selector.target)?.total;
                    let count = self.as_usize(value, "selector")?;
                    self.select_set_lowest(elements, count, only_kept)
                }
                SelectorKind::GreaterThan => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_set_value(elements, |element| element > value, only_kept)
                }
                SelectorKind::GreaterThanOrEqual => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_set_value(elements, |element| element >= value, only_kept)
                }
                SelectorKind::LessThan => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_set_value(elements, |element| element < value, only_kept)
                }
                SelectorKind::LessThanOrEqual => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_set_value(elements, |element| element <= value, only_kept)
                }
                SelectorKind::EqualTo => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_set_value(
                        elements,
                        |element| (element - value).abs() <= EPSILON,
                        only_kept,
                    )
                }
                SelectorKind::NotEqual => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_set_value(
                        elements,
                        |element| (element - value).abs() > EPSILON,
                        only_kept,
                    )
                }
                SelectorKind::Literal => {
                    let value = self.eval(&selector.target)?.total;
                    self.select_set_value(
                        elements,
                        |element| (element - value).abs() <= EPSILON,
                        only_kept,
                    )
                }
            }?;
            selected.extend(indices.drain(..));
        }
        let mut collected: Vec<_> = selected.into_iter().collect();
        collected.sort_unstable();
        Ok(collected)
    }

    fn select_highest(&self, dice: &[DieResult], count: usize) -> Result<Vec<usize>> {
        let mut indices: Vec<_> = dice
            .iter()
            .enumerate()
            .filter(|(_, die)| die.kept)
            .map(|(idx, _)| idx)
            .collect();
        indices.sort_by(|a, b| self.compare_desc(&dice[*a].value, &dice[*b].value));
        indices.truncate(count.min(indices.len()));
        Ok(indices)
    }

    fn select_lowest(&self, dice: &[DieResult], count: usize) -> Result<Vec<usize>> {
        let mut indices: Vec<_> = dice
            .iter()
            .enumerate()
            .filter(|(_, die)| die.kept)
            .map(|(idx, _)| idx)
            .collect();
        indices.sort_by(|a, b| self.compare_asc(&dice[*a].value, &dice[*b].value));
        indices.truncate(count.min(indices.len()));
        Ok(indices)
    }

    fn select_value<F>(&self, dice: &[DieResult], predicate: F) -> Result<Vec<usize>>
    where
        F: Fn(f64) -> bool,
    {
        Ok(dice
            .iter()
            .enumerate()
            .filter(|(_, die)| die.kept && predicate(die.value))
            .map(|(idx, _)| idx)
            .collect())
    }

    fn select_set_highest<T>(
        &self,
        elements: &[SetElement<T>],
        count: usize,
        only_kept: bool,
    ) -> Result<Vec<usize>> {
        let mut indices: Vec<_> = elements
            .iter()
            .enumerate()
            .filter(|(_, element)| !only_kept || element.kept)
            .map(|(idx, _)| idx)
            .collect();
        indices.sort_by(|a, b| {
            self.compare_desc(&elements[*a].value.total, &elements[*b].value.total)
        });
        indices.truncate(count.min(indices.len()));
        Ok(indices)
    }

    fn select_set_lowest<T>(
        &self,
        elements: &[SetElement<T>],
        count: usize,
        only_kept: bool,
    ) -> Result<Vec<usize>> {
        let mut indices: Vec<_> = elements
            .iter()
            .enumerate()
            .filter(|(_, element)| !only_kept || element.kept)
            .map(|(idx, _)| idx)
            .collect();
        indices
            .sort_by(|a, b| self.compare_asc(&elements[*a].value.total, &elements[*b].value.total));
        indices.truncate(count.min(indices.len()));
        Ok(indices)
    }

    fn select_set_value<T, F>(
        &self,
        elements: &[SetElement<T>],
        predicate: F,
        only_kept: bool,
    ) -> Result<Vec<usize>>
    where
        F: Fn(f64) -> bool,
    {
        Ok(elements
            .iter()
            .enumerate()
            .filter(|(_, element)| (!only_kept || element.kept) && predicate(element.value.total))
            .map(|(idx, _)| idx)
            .collect())
    }

    fn compare_desc(&self, a: &f64, b: &f64) -> Ordering {
        b.partial_cmp(a).unwrap_or(Ordering::Equal)
    }

    fn compare_asc(&self, a: &f64, b: &f64) -> Ordering {
        a.partial_cmp(b).unwrap_or(Ordering::Equal)
    }
}
