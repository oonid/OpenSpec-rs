use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub raw_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Requirement {
    pub text: String,
    pub scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    pub name: String,
    pub overview: String,
    pub requirements: Vec<Requirement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SpecMetadata>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecMetadata {
    pub version: String,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeltaOperation {
    Added,
    Modified,
    Removed,
    Renamed,
}

impl std::fmt::Display for DeltaOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeltaOperation::Added => write!(f, "ADDED"),
            DeltaOperation::Modified => write!(f, "MODIFIED"),
            DeltaOperation::Removed => write!(f, "REMOVED"),
            DeltaOperation::Renamed => write!(f, "RENAMED"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequirementBlock {
    pub header_line: String,
    pub name: String,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenamePair {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionPresence {
    pub added: bool,
    pub modified: bool,
    pub removed: bool,
    pub renamed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeltaPlan {
    pub added: Vec<RequirementBlock>,
    pub modified: Vec<RequirementBlock>,
    pub removed: Vec<String>,
    pub renamed: Vec<RenamePair>,
    pub section_presence: SectionPresence,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub level: usize,
    pub title: String,
    pub content: String,
    pub children: Vec<Section>,
}

pub struct SpecParser {
    lines: Vec<String>,
}

impl SpecParser {
    pub fn new(content: &str) -> Self {
        let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
        Self {
            lines: normalized.split('\n').map(String::from).collect(),
        }
    }

    pub fn parse_spec(&mut self, name: &str) -> Result<Spec, String> {
        let sections = self.parse_sections();
        let purpose = self
            .find_section(&sections, "Purpose")
            .map(|s| s.content.clone())
            .unwrap_or_default();

        let requirements_section = self.find_section(&sections, "Requirements");

        if purpose.trim().is_empty() {
            return Err("Spec must have a Purpose section".to_string());
        }

        let requirements = if let Some(req_section) = requirements_section {
            self.parse_requirements(req_section)
        } else {
            return Err("Spec must have a Requirements section".to_string());
        };

        Ok(Spec {
            name: name.to_string(),
            overview: purpose.trim().to_string(),
            requirements,
            metadata: Some(SpecMetadata {
                version: "1.0.0".to_string(),
                format: "openspec".to_string(),
                source_path: None,
            }),
        })
    }

    fn find_section<'a>(&self, sections: &'a [Section], title: &str) -> Option<&'a Section> {
        let target = title.to_lowercase();
        for section in sections {
            if section.title.to_lowercase() == target {
                return Some(section);
            }
            if let Some(child) = self.find_section(&section.children, title) {
                return Some(child);
            }
        }
        None
    }

    fn parse_sections(&self) -> Vec<Section> {
        let mut all_sections: Vec<(usize, Section)> = Vec::new();
        let mut parent_indices: Vec<Option<usize>> = Vec::new();
        let mut stack: Vec<usize> = Vec::new();

        for i in 0..self.lines.len() {
            let line = &self.lines[i];
            if let Some((level, title)) = parse_header(line) {
                let content = self.get_content_until_next_header(i + 1, level);
                let section = Section {
                    level,
                    title: title.clone(),
                    content,
                    children: Vec::new(),
                };

                let section_idx = all_sections.len();
                all_sections.push((level, section));
                parent_indices.push(None);

                while !stack.is_empty() && all_sections[*stack.last().unwrap()].0 >= level {
                    stack.pop();
                }

                if let Some(&parent_idx) = stack.last() {
                    parent_indices[section_idx] = Some(parent_idx);
                }

                stack.push(section_idx);
            }
        }

        let n = all_sections.len();
        let mut children_map: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (child_idx, parent_opt) in parent_indices.iter().enumerate() {
            if let Some(parent_idx) = parent_opt {
                children_map[*parent_idx].push(child_idx);
            }
        }

        fn build_section(
            idx: usize,
            sections: &mut [(usize, Section)],
            children: &[Vec<usize>],
        ) -> Section {
            let (_, ref mut section) = sections[idx];
            let section_clone = section.clone();
            let mut result = Section {
                level: section_clone.level,
                title: section_clone.title.clone(),
                content: section_clone.content.clone(),
                children: Vec::new(),
            };

            for &child_idx in &children[idx] {
                result
                    .children
                    .push(build_section(child_idx, sections, children));
            }

            result
        }

        let mut root_sections = Vec::new();
        for (idx, parent_opt) in parent_indices.iter().enumerate() {
            if parent_opt.is_none() {
                root_sections.push(build_section(idx, &mut all_sections, &children_map));
            }
        }

        root_sections
    }

    fn get_content_until_next_header(&self, start_line: usize, current_level: usize) -> String {
        let mut content_lines: Vec<String> = Vec::new();

        for i in start_line..self.lines.len() {
            let line = &self.lines[i];
            if let Some((level, _)) = parse_header(line) {
                if level <= current_level {
                    break;
                }
            }
            content_lines.push(line.clone());
        }

        content_lines.join("\n").trim().to_string()
    }

    fn parse_requirements(&self, section: &Section) -> Vec<Requirement> {
        let mut requirements: Vec<Requirement> = Vec::new();

        for child in &section.children {
            let text = if child.content.trim().is_empty() {
                child.title.clone()
            } else {
                let lines: Vec<&str> = child.content.split('\n').collect();
                let content_before_children: String = lines
                    .iter()
                    .take_while(|l| !l.trim().starts_with('#'))
                    .cloned()
                    .collect::<Vec<&str>>()
                    .join("\n");

                let trimmed = content_before_children.trim();
                if trimmed.is_empty() {
                    child.title.clone()
                } else {
                    trimmed
                        .lines()
                        .next()
                        .unwrap_or(&child.title)
                        .trim()
                        .to_string()
                }
            };

            let scenarios = self.parse_scenarios(child);

            requirements.push(Requirement { text, scenarios });
        }

        requirements
    }

    fn parse_scenarios(&self, requirement_section: &Section) -> Vec<Scenario> {
        let mut scenarios: Vec<Scenario> = Vec::new();

        for scenario_section in &requirement_section.children {
            if !scenario_section.content.trim().is_empty() {
                scenarios.push(Scenario {
                    raw_text: scenario_section.content.clone(),
                });
            }
        }

        scenarios
    }
}

fn parse_header(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim();
    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
    if (1..=6).contains(&hash_count) {
        let title = trimmed[hash_count..].trim();
        if !title.is_empty() {
            return Some((hash_count, title.to_string()));
        }
    }
    None
}

pub fn parse_delta_spec(content: &str) -> DeltaPlan {
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let sections = split_top_level_sections(&normalized);

    let added_lookup = get_section_case_insensitive(&sections, "ADDED Requirements");
    let modified_lookup = get_section_case_insensitive(&sections, "MODIFIED Requirements");
    let removed_lookup = get_section_case_insensitive(&sections, "REMOVED Requirements");
    let renamed_lookup = get_section_case_insensitive(&sections, "RENAMED Requirements");

    let added = parse_requirement_blocks_from_section(&added_lookup.body);
    let modified = parse_requirement_blocks_from_section(&modified_lookup.body);
    let removed = parse_removed_names(&removed_lookup.body);
    let renamed = parse_renamed_pairs(&renamed_lookup.body);

    DeltaPlan {
        added,
        modified,
        removed,
        renamed,
        section_presence: SectionPresence {
            added: added_lookup.found,
            modified: modified_lookup.found,
            removed: removed_lookup.found,
            renamed: renamed_lookup.found,
        },
    }
}

fn split_top_level_sections(content: &str) -> HashMap<String, String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result: HashMap<String, String> = HashMap::new();
    let mut indices: Vec<(String, usize)> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if let Some((2, title)) = parse_header(line) {
            indices.push((title, i));
        }
    }

    for i in 0..indices.len() {
        let (title, start_idx) = &indices[i];
        let end_idx = if i + 1 < indices.len() {
            indices[i + 1].1
        } else {
            lines.len()
        };
        let body: String = lines[*start_idx + 1..end_idx].join("\n");
        result.insert(title.clone(), body);
    }

    result
}

fn get_section_case_insensitive(
    sections: &HashMap<String, String>,
    desired: &str,
) -> SectionLookup {
    let target = desired.to_lowercase();
    for (title, body) in sections {
        if title.to_lowercase() == target {
            return SectionLookup {
                body: body.clone(),
                found: true,
            };
        }
    }
    SectionLookup {
        body: String::new(),
        found: false,
    }
}

struct SectionLookup {
    body: String,
    found: bool,
}

const REQUIREMENT_HEADER_REGEX: &str = r"^###\s*Requirement:\s*(.+?)\s*$";

fn parse_requirement_blocks_from_section(section_body: &str) -> Vec<RequirementBlock> {
    if section_body.trim().is_empty() {
        return Vec::new();
    }

    let lines: Vec<&str> = section_body.lines().collect();
    let mut blocks: Vec<RequirementBlock> = Vec::new();
    let mut i = 0;

    let re = regex::Regex::new(REQUIREMENT_HEADER_REGEX).unwrap();

    while i < lines.len() {
        while i < lines.len() && !re.is_match(lines[i]) {
            i += 1;
        }
        if i >= lines.len() {
            break;
        }

        let header_line = lines[i];
        let caps = re.captures(header_line).unwrap();
        let name = caps[1].trim().to_string();

        let mut buf: Vec<&str> = vec![header_line];
        i += 1;

        while i < lines.len() && !re.is_match(lines[i]) && !lines[i].trim().starts_with("## ") {
            buf.push(lines[i]);
            i += 1;
        }

        blocks.push(RequirementBlock {
            header_line: header_line.to_string(),
            name,
            raw: buf.join("\n").trim_end().to_string(),
        });
    }

    blocks
}

fn parse_removed_names(section_body: &str) -> Vec<String> {
    if section_body.trim().is_empty() {
        return Vec::new();
    }

    let mut names: Vec<String> = Vec::new();
    let re = regex::Regex::new(REQUIREMENT_HEADER_REGEX).unwrap();
    let bullet_re = regex::Regex::new(r"^\s*-\s*`?###\s*Requirement:\s*(.+?)`?\s*$").unwrap();

    for line in section_body.lines() {
        if let Some(caps) = re.captures(line) {
            names.push(caps[1].trim().to_string());
        } else if let Some(caps) = bullet_re.captures(line) {
            names.push(caps[1].trim().to_string());
        }
    }

    names
}

fn parse_renamed_pairs(section_body: &str) -> Vec<RenamePair> {
    if section_body.trim().is_empty() {
        return Vec::new();
    }

    let mut pairs: Vec<RenamePair> = Vec::new();
    let from_re =
        regex::Regex::new(r"^\s*-?\s*FROM:\s*`?###\s*Requirement:\s*(.+?)`?\s*$").unwrap();
    let to_re = regex::Regex::new(r"^\s*-?\s*TO:\s*`?###\s*Requirement:\s*(.+?)`?\s*$").unwrap();

    let mut current: Option<(Option<String>, Option<String>)> = None;

    for line in section_body.lines() {
        if let Some(caps) = from_re.captures(line) {
            if current.is_none() {
                current = Some((None, None));
            }
            current.as_mut().unwrap().0 = Some(caps[1].trim().to_string());
        } else if let Some(caps) = to_re.captures(line) {
            if current.is_none() {
                current = Some((None, None));
            }
            current.as_mut().unwrap().1 = Some(caps[1].trim().to_string());

            if let Some((Some(from), Some(to))) = current.take() {
                pairs.push(RenamePair { from, to });
            }
        }
    }

    pairs
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequirementsSectionParts {
    pub before: String,
    pub header_line: String,
    pub preamble: String,
    pub body_blocks: Vec<RequirementBlock>,
    pub after: String,
}

pub fn extract_requirements_section(content: &str) -> RequirementsSectionParts {
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = normalized.lines().collect();

    let req_header_idx = lines
        .iter()
        .position(|l| l.trim().eq_ignore_ascii_case("## Requirements"));

    if req_header_idx.is_none() {
        let before = content.trim_end();
        return RequirementsSectionParts {
            before: if before.is_empty() {
                String::new()
            } else {
                format!("{}\n\n", before)
            },
            header_line: "## Requirements".to_string(),
            preamble: String::new(),
            body_blocks: Vec::new(),
            after: "\n".to_string(),
        };
    }

    let req_header_idx = req_header_idx.unwrap();
    let header_line = lines[req_header_idx].to_string();

    let mut end_idx = lines.len();
    for (i, line) in lines.iter().enumerate().skip(req_header_idx + 1) {
        if line.trim().starts_with("## ") {
            end_idx = i;
            break;
        }
    }

    let before = if req_header_idx > 0 {
        format!("{}\n", lines[..req_header_idx].join("\n").trim_end())
    } else {
        String::new()
    };

    let section_body: Vec<&str> = lines[req_header_idx + 1..end_idx].to_vec();

    let (preamble_lines, body_lines) = {
        let mut preamble = Vec::new();
        let mut body = Vec::new();
        let mut in_body = false;

        for line in section_body {
            if line.trim().starts_with("### Requirement:") {
                in_body = true;
            }
            if in_body {
                body.push(line);
            } else {
                preamble.push(line);
            }
        }
        (preamble, body)
    };

    let body_blocks = parse_requirement_blocks_from_lines(&body_lines);
    let preamble = preamble_lines.join("\n").trim_end().to_string();
    let after_lines = &lines[end_idx..];
    let after = if after_lines.is_empty() {
        "\n".to_string()
    } else {
        format!("\n{}", after_lines.join("\n"))
    };

    RequirementsSectionParts {
        before,
        header_line,
        preamble,
        body_blocks,
        after,
    }
}

fn parse_requirement_blocks_from_lines(lines: &[&str]) -> Vec<RequirementBlock> {
    let mut blocks: Vec<RequirementBlock> = Vec::new();
    let re = regex::Regex::new(REQUIREMENT_HEADER_REGEX).unwrap();
    let mut i = 0;

    while i < lines.len() {
        while i < lines.len() && !re.is_match(lines[i]) {
            i += 1;
        }
        if i >= lines.len() {
            break;
        }

        let header_line = lines[i];
        let caps = re.captures(header_line).unwrap();
        let name = caps[1].trim().to_string();

        let mut buf: Vec<&str> = vec![header_line];
        i += 1;

        while i < lines.len() && !re.is_match(lines[i]) && !lines[i].trim().starts_with("## ") {
            buf.push(lines[i]);
            i += 1;
        }

        blocks.push(RequirementBlock {
            header_line: header_line.to_string(),
            name,
            raw: buf.join("\n").trim_end().to_string(),
        });
    }

    blocks
}

pub fn build_spec_skeleton(spec_folder_name: &str, change_name: &str) -> String {
    format!(
        "# {} Specification\n\n## Purpose\nTBD - created by archiving change {}. Update Purpose after archive.\n\n## Requirements\n",
        spec_folder_name, change_name
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct MergeResult {
    pub rebuilt: String,
    pub counts: MergeCounts,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MergeCounts {
    pub added: usize,
    pub modified: usize,
    pub removed: usize,
    pub renamed: usize,
}

pub fn merge_delta_plan(
    target_content: &str,
    plan: &DeltaPlan,
    spec_name: &str,
    change_name: &str,
) -> Result<MergeResult, String> {
    let mut added_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for add in &plan.added {
        let name = normalize_requirement_name(&add.name);
        if added_names.contains(&name) {
            return Err(format!(
                "{} validation failed - duplicate requirement in ADDED for header \"### Requirement: {}\"",
                spec_name, add.name
            ));
        }
        added_names.insert(name);
    }

    let mut modified_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for m in &plan.modified {
        let name = normalize_requirement_name(&m.name);
        if modified_names.contains(&name) {
            return Err(format!(
                "{} validation failed - duplicate requirement in MODIFIED for header \"### Requirement: {}\"",
                spec_name, m.name
            ));
        }
        modified_names.insert(name);
    }

    let mut removed_names_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    for rem in &plan.removed {
        let name = normalize_requirement_name(rem);
        if removed_names_set.contains(&name) {
            return Err(format!(
                "{} validation failed - duplicate requirement in REMOVED for header \"### Requirement: {}\"",
                spec_name, rem
            ));
        }
        removed_names_set.insert(name);
    }

    let mut renamed_from_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut renamed_to_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    for rp in &plan.renamed {
        let from = normalize_requirement_name(&rp.from);
        let to = normalize_requirement_name(&rp.to);
        if renamed_from_set.contains(&from) {
            return Err(format!(
                "{} validation failed - duplicate FROM in RENAMED for header \"### Requirement: {}\"",
                spec_name, rp.from
            ));
        }
        if renamed_to_set.contains(&to) {
            return Err(format!(
                "{} validation failed - duplicate TO in RENAMED for header \"### Requirement: {}\"",
                spec_name, rp.to
            ));
        }
        renamed_from_set.insert(from);
        renamed_to_set.insert(to);
    }

    for n in &modified_names {
        if removed_names_set.contains(n) {
            return Err(format!(
                "{} validation failed - requirement present in both MODIFIED and REMOVED for \"{}\"",
                spec_name, n
            ));
        }
        if added_names.contains(n) {
            return Err(format!(
                "{} validation failed - requirement present in both MODIFIED and ADDED for \"{}\"",
                spec_name, n
            ));
        }
    }
    for n in &added_names {
        if removed_names_set.contains(n) {
            return Err(format!(
                "{} validation failed - requirement present in both ADDED and REMOVED for \"{}\"",
                spec_name, n
            ));
        }
    }

    for rp in &plan.renamed {
        let to_norm = normalize_requirement_name(&rp.to);
        if modified_names.contains(&to_norm) {
            return Err(format!(
                "{} validation failed - when a rename exists, MODIFIED must reference the NEW header \"### Requirement: {}\"",
                spec_name, rp.to
            ));
        }
        if added_names.contains(&to_norm) {
            return Err(format!(
                "{} validation failed - RENAMED TO header collides with ADDED for \"### Requirement: {}\"",
                spec_name, rp.to
            ));
        }
    }

    let has_any_delta = !plan.added.is_empty()
        || !plan.modified.is_empty()
        || !plan.removed.is_empty()
        || !plan.renamed.is_empty();
    if !has_any_delta {
        return Err(format!(
            "Delta parsing found no operations for {}. Provide ADDED/MODIFIED/REMOVED/RENAMED sections in change spec.",
            spec_name
        ));
    }

    let (parts, is_new_spec) = if target_content.trim().is_empty() {
        if !plan.modified.is_empty() || !plan.renamed.is_empty() {
            return Err(format!(
                "{}: target spec does not exist; only ADDED requirements are allowed for new specs. MODIFIED and RENAMED operations require an existing spec.",
                spec_name
            ));
        }
        (
            extract_requirements_section(&build_spec_skeleton(spec_name, change_name)),
            true,
        )
    } else {
        (extract_requirements_section(target_content), false)
    };

    let mut name_to_block: HashMap<String, RequirementBlock> = HashMap::new();
    for block in &parts.body_blocks {
        name_to_block.insert(normalize_requirement_name(&block.name), block.clone());
    }

    let mut rename_map: HashMap<String, String> = HashMap::new();
    let mut counts = MergeCounts::default();

    for rp in &plan.renamed {
        let from = normalize_requirement_name(&rp.from);
        let to = normalize_requirement_name(&rp.to);
        if !name_to_block.contains_key(&from) {
            return Err(format!(
                "{} RENAMED failed for header \"### Requirement: {}\" - source not found",
                spec_name, rp.from
            ));
        }
        if name_to_block.contains_key(&to) {
            return Err(format!(
                "{} RENAMED failed for header \"### Requirement: {}\" - target already exists",
                spec_name, rp.to
            ));
        }
        let block = name_to_block.get(&from).unwrap().clone();
        let new_header = format!("### Requirement: {}", to);
        let raw_lines: Vec<&str> = block.raw.lines().collect();
        let mut new_raw_lines: Vec<String> = vec![new_header.clone()];
        for line in &raw_lines[1..] {
            new_raw_lines.push(line.to_string());
        }
        let renamed_block = RequirementBlock {
            header_line: new_header,
            name: to.clone(),
            raw: new_raw_lines.join("\n"),
        };
        name_to_block.remove(&from);
        name_to_block.insert(to.clone(), renamed_block);
        rename_map.insert(from, to);
        counts.renamed += 1;
    }

    for name in &plan.removed {
        let key = normalize_requirement_name(name);
        if !name_to_block.contains_key(&key) {
            if !is_new_spec {
                return Err(format!(
                    "{} REMOVED failed for header \"### Requirement: {}\" - not found",
                    spec_name, name
                ));
            }
            continue;
        }
        name_to_block.remove(&key);
        counts.removed += 1;
    }

    for m in &plan.modified {
        let key = normalize_requirement_name(&m.name);
        if !name_to_block.contains_key(&key) {
            return Err(format!(
                "{} MODIFIED failed for header \"### Requirement: {}\" - not found",
                spec_name, m.name
            ));
        }
        name_to_block.insert(key, m.clone());
        counts.modified += 1;
    }

    for add in &plan.added {
        let key = normalize_requirement_name(&add.name);
        if name_to_block.contains_key(&key) {
            return Err(format!(
                "{} ADDED failed for header \"### Requirement: {}\" - already exists",
                spec_name, add.name
            ));
        }
        name_to_block.insert(key, add.clone());
        counts.added += 1;
    }

    let mut kept_order: Vec<RequirementBlock> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for block in &parts.body_blocks {
        let key = normalize_requirement_name(&block.name);
        let lookup_key = rename_map.get(&key).cloned().unwrap_or_else(|| key.clone());
        if let Some(replacement) = name_to_block.get(&lookup_key) {
            kept_order.push(replacement.clone());
            seen.insert(lookup_key);
        }
    }
    for (key, block) in &name_to_block {
        if !seen.contains(key) {
            kept_order.push(block.clone());
        }
    }

    let req_body_parts: Vec<String> = vec![
        if !parts.preamble.is_empty() {
            Some(parts.preamble.trim_end().to_string())
        } else {
            None
        },
        if !kept_order.is_empty() {
            Some(
                kept_order
                    .iter()
                    .map(|b| b.raw.as_str())
                    .collect::<Vec<_>>()
                    .join("\n\n"),
            )
        } else {
            None
        },
    ]
    .into_iter()
    .flatten()
    .collect();

    let req_body = req_body_parts.join("\n\n").trim_end().to_string();

    let rebuilt_parts: Vec<&str> = vec![
        parts.before.trim_end(),
        &parts.header_line,
        &req_body,
        parts.after.trim_start(),
    ];
    let rebuilt = rebuilt_parts
        .iter()
        .filter(|s| !s.is_empty())
        .cloned()
        .collect::<Vec<&str>>()
        .join("\n")
        .replace("\n\n\n", "\n\n");

    Ok(MergeResult { rebuilt, counts })
}

fn normalize_requirement_name(name: &str) -> String {
    name.trim().to_string()
}

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct SpecInfo {
    pub name: String,
    pub path: PathBuf,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpecUpdate {
    pub source: PathBuf,
    pub target: PathBuf,
    pub spec_name: String,
    pub target_exists: bool,
}

pub fn find_specs(specs_dir: &Path) -> Vec<SpecInfo> {
    let mut specs = Vec::new();

    if !specs_dir.exists() || !specs_dir.is_dir() {
        return specs;
    }

    if let Ok(entries) = std::fs::read_dir(specs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let spec_file = path.join("spec.md");
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                let exists = spec_file.exists();
                specs.push(SpecInfo {
                    name,
                    path: spec_file,
                    exists,
                });
            }
        }
    }

    specs.sort_by(|a, b| a.name.cmp(&b.name));
    specs
}

pub fn find_change_specs(change_dir: &Path) -> Vec<SpecInfo> {
    let specs_subdir = change_dir.join("specs");
    find_specs(&specs_subdir)
}

pub fn find_spec_updates(change_dir: &Path, main_specs_dir: &Path) -> Vec<SpecUpdate> {
    let mut updates = Vec::new();
    let change_specs_dir = change_dir.join("specs");

    if !change_specs_dir.exists() || !change_specs_dir.is_dir() {
        return updates;
    }

    if let Ok(entries) = std::fs::read_dir(&change_specs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    let spec_name = name.to_string_lossy().to_string();
                    let source = path.join("spec.md");
                    let target = main_specs_dir.join(&spec_name).join("spec.md");

                    if source.exists() {
                        let target_exists = target.exists();
                        updates.push(SpecUpdate {
                            source,
                            target,
                            spec_name,
                            target_exists,
                        });
                    }
                }
            }
        }
    }

    updates.sort_by(|a, b| a.spec_name.cmp(&b.spec_name));
    updates
}

pub fn glob_has_matches(base_dir: &Path, pattern: &str) -> bool {
    let normalized = pattern.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').collect();

    let mut dir_parts: Vec<&str> = Vec::new();
    let mut pattern_part: Option<&str> = None;

    for part in parts {
        if part.contains('*') {
            pattern_part = Some(part);
            break;
        }
        dir_parts.push(part);
    }

    let base = if dir_parts.is_empty() {
        base_dir.to_path_buf()
    } else {
        base_dir.join(dir_parts.join("/"))
    };

    if !base.exists() || !base.is_dir() {
        return false;
    }

    let pattern_str = pattern_part.unwrap_or("*");
    let expected_ext = pattern_str
        .strip_prefix('*')
        .filter(|ext| ext.starts_with('.'));

    fn check_dir(dir: &Path, ext_filter: Option<&str>, recursive: bool) -> bool {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && recursive {
                    if check_dir(&path, ext_filter, recursive) {
                        return true;
                    }
                } else if path.is_file() {
                    if let Some(ext) = ext_filter {
                        if path.extension().map(|e| e.to_string_lossy()).as_deref()
                            == Some(&ext[1..])
                        {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
            }
        }
        false
    }

    let recursive = normalized.contains("**");
    check_dir(&base, expected_ext, recursive)
}

pub fn artifact_output_exists(change_dir: &Path, generates: &str) -> bool {
    let normalized = generates.replace('\\', "/");
    let full_path = change_dir.join(&normalized);

    if normalized.contains('*') {
        glob_has_matches(change_dir, generates)
    } else {
        full_path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec() {
        let content = r#"# test-spec Specification

## Purpose
This is the purpose of the spec.

## Requirements
### Requirement: First Requirement
The system SHALL do something.

#### Scenario: First scenario
- **WHEN** something happens
- **THEN** something else happens
"#;

        let mut parser = SpecParser::new(content);
        let spec = parser.parse_spec("test-spec").unwrap();

        assert_eq!(spec.name, "test-spec");
        assert_eq!(spec.overview, "This is the purpose of the spec.");
        assert_eq!(spec.requirements.len(), 1);
        assert_eq!(spec.requirements[0].text, "The system SHALL do something.");
        assert_eq!(spec.requirements[0].scenarios.len(), 1);
    }

    #[test]
    fn test_parse_spec_missing_purpose() {
        let content = r#"# test-spec Specification

## Requirements
### Requirement: First Requirement
The system SHALL do something.
"#;

        let mut parser = SpecParser::new(content);
        let result = parser.parse_spec("test-spec");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Purpose"));
    }

    #[test]
    fn test_parse_spec_missing_requirements() {
        let content = r#"# test-spec Specification

## Purpose
This is the purpose.
"#;

        let mut parser = SpecParser::new(content);
        let result = parser.parse_spec("test-spec");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Requirements"));
    }

    #[test]
    fn test_parse_delta_spec_added() {
        let content = r#"## ADDED Requirements

### Requirement: New Feature
The system SHALL provide new functionality.

#### Scenario: Test new feature
- **WHEN** the feature is used
- **THEN** it works
"#;

        let plan = parse_delta_spec(content);

        assert!(plan.section_presence.added);
        assert!(!plan.section_presence.modified);
        assert!(!plan.section_presence.removed);
        assert!(!plan.section_presence.renamed);
        assert_eq!(plan.added.len(), 1);
        assert_eq!(plan.added[0].name, "New Feature");
    }

    #[test]
    fn test_parse_delta_spec_modified() {
        let content = r#"## MODIFIED Requirements

### Requirement: Existing Feature
The system SHALL provide updated functionality.
"#;

        let plan = parse_delta_spec(content);

        assert!(!plan.section_presence.added);
        assert!(plan.section_presence.modified);
        assert_eq!(plan.modified.len(), 1);
        assert_eq!(plan.modified[0].name, "Existing Feature");
    }

    #[test]
    fn test_parse_delta_spec_removed() {
        let content = r#"## REMOVED Requirements

### Requirement: Old Feature
"#;

        let plan = parse_delta_spec(content);

        assert!(plan.section_presence.removed);
        assert_eq!(plan.removed.len(), 1);
        assert_eq!(plan.removed[0], "Old Feature");
    }

    #[test]
    fn test_parse_delta_spec_renamed() {
        let content = r#"## RENAMED Requirements

- FROM: `### Requirement: Old Name`
- TO: `### Requirement: New Name`
"#;

        let plan = parse_delta_spec(content);

        assert!(plan.section_presence.renamed);
        assert_eq!(plan.renamed.len(), 1);
        assert_eq!(plan.renamed[0].from, "Old Name");
        assert_eq!(plan.renamed[0].to, "New Name");
    }

    #[test]
    fn test_parse_delta_spec_multiple_sections() {
        let content = r#"## ADDED Requirements

### Requirement: New Feature
The system SHALL do something new.

## MODIFIED Requirements

### Requirement: Changed Feature
The system SHALL do something differently.

## REMOVED Requirements

### Requirement: Deprecated Feature
"#;

        let plan = parse_delta_spec(content);

        assert!(plan.section_presence.added);
        assert!(plan.section_presence.modified);
        assert!(plan.section_presence.removed);
        assert_eq!(plan.added.len(), 1);
        assert_eq!(plan.modified.len(), 1);
        assert_eq!(plan.removed.len(), 1);
    }

    #[test]
    fn test_parse_delta_spec_case_insensitive() {
        let content = r#"## added requirements

### Requirement: New Feature
"#;

        let plan = parse_delta_spec(content);
        assert!(plan.section_presence.added);
    }

    #[test]
    fn test_parse_requirement_with_multiple_scenarios() {
        let content = r#"# test-spec Specification

## Purpose
Test purpose.

## Requirements
### Requirement: Multi-scenario
The system SHALL handle multiple scenarios.

#### Scenario: First scenario
- **WHEN** first condition
- **THEN** first result

#### Scenario: Second scenario
- **WHEN** second condition
- **THEN** second result
"#;

        let mut parser = SpecParser::new(content);
        let spec = parser.parse_spec("test-spec").unwrap();

        assert_eq!(spec.requirements[0].scenarios.len(), 2);
    }

    #[test]
    fn test_merge_add_requirements() {
        let target = r#"# test-spec Specification

## Purpose
Test purpose.

## Requirements
### Requirement: Existing
Some existing requirement.
"#;
        let plan = DeltaPlan {
            added: vec![RequirementBlock {
                header_line: "### Requirement: New Feature".to_string(),
                name: "New Feature".to_string(),
                raw: "### Requirement: New Feature\nThe system SHALL do something new.".to_string(),
            }],
            modified: vec![],
            removed: vec![],
            renamed: vec![],
            section_presence: SectionPresence {
                added: true,
                modified: false,
                removed: false,
                renamed: false,
            },
        };

        let result = merge_delta_plan(target, &plan, "test-spec", "test-change").unwrap();
        assert_eq!(result.counts.added, 1);
        assert!(result.rebuilt.contains("### Requirement: Existing"));
        assert!(result.rebuilt.contains("### Requirement: New Feature"));
    }

    #[test]
    fn test_merge_modify_requirements() {
        let target = r#"# test-spec Specification

## Purpose
Test purpose.

## Requirements
### Requirement: Existing
Some existing requirement.
"#;
        let plan = DeltaPlan {
            added: vec![],
            modified: vec![RequirementBlock {
                header_line: "### Requirement: Existing".to_string(),
                name: "Existing".to_string(),
                raw: "### Requirement: Existing\nUpdated requirement text.".to_string(),
            }],
            removed: vec![],
            renamed: vec![],
            section_presence: SectionPresence {
                added: false,
                modified: true,
                removed: false,
                renamed: false,
            },
        };

        let result = merge_delta_plan(target, &plan, "test-spec", "test-change").unwrap();
        assert_eq!(result.counts.modified, 1);
        assert!(result.rebuilt.contains("Updated requirement text."));
        assert!(!result.rebuilt.contains("Some existing requirement."));
    }

    #[test]
    fn test_merge_remove_requirements() {
        let target = r#"# test-spec Specification

## Purpose
Test purpose.

## Requirements
### Requirement: Keep This
Keep this requirement.

### Requirement: Remove This
Remove this requirement.
"#;
        let plan = DeltaPlan {
            added: vec![],
            modified: vec![],
            removed: vec!["Remove This".to_string()],
            renamed: vec![],
            section_presence: SectionPresence {
                added: false,
                modified: false,
                removed: true,
                renamed: false,
            },
        };

        let result = merge_delta_plan(target, &plan, "test-spec", "test-change").unwrap();
        assert_eq!(result.counts.removed, 1);
        assert!(result.rebuilt.contains("### Requirement: Keep This"));
        assert!(!result.rebuilt.contains("### Requirement: Remove This"));
    }

    #[test]
    fn test_merge_rename_requirements() {
        let target = r#"# test-spec Specification

## Purpose
Test purpose.

## Requirements
### Requirement: Old Name
Some requirement content.
"#;
        let plan = DeltaPlan {
            added: vec![],
            modified: vec![],
            removed: vec![],
            renamed: vec![RenamePair {
                from: "Old Name".to_string(),
                to: "New Name".to_string(),
            }],
            section_presence: SectionPresence {
                added: false,
                modified: false,
                removed: false,
                renamed: true,
            },
        };

        let result = merge_delta_plan(target, &plan, "test-spec", "test-change").unwrap();
        assert_eq!(result.counts.renamed, 1);
        assert!(result.rebuilt.contains("### Requirement: New Name"));
        assert!(!result.rebuilt.contains("### Requirement: Old Name"));
        assert!(result.rebuilt.contains("Some requirement content."));
    }

    #[test]
    fn test_merge_new_spec_added_only() {
        let plan = DeltaPlan {
            added: vec![RequirementBlock {
                header_line: "### Requirement: First Feature".to_string(),
                name: "First Feature".to_string(),
                raw: "### Requirement: First Feature\nThe system SHALL do something.".to_string(),
            }],
            modified: vec![],
            removed: vec![],
            renamed: vec![],
            section_presence: SectionPresence {
                added: true,
                modified: false,
                removed: false,
                renamed: false,
            },
        };

        let result = merge_delta_plan("", &plan, "new-spec", "test-change").unwrap();
        assert_eq!(result.counts.added, 1);
        assert!(result.rebuilt.contains("# new-spec Specification"));
        assert!(result
            .rebuilt
            .contains("created by archiving change test-change"));
        assert!(result.rebuilt.contains("### Requirement: First Feature"));
    }

    #[test]
    fn test_merge_new_spec_with_removed_ignored() {
        let plan = DeltaPlan {
            added: vec![RequirementBlock {
                header_line: "### Requirement: New Feature".to_string(),
                name: "New Feature".to_string(),
                raw: "### Requirement: New Feature\nNew content.".to_string(),
            }],
            modified: vec![],
            removed: vec!["Nonexistent".to_string()],
            renamed: vec![],
            section_presence: SectionPresence {
                added: true,
                modified: false,
                removed: true,
                renamed: false,
            },
        };

        let result = merge_delta_plan("", &plan, "new-spec", "test-change").unwrap();
        assert_eq!(result.counts.added, 1);
        assert_eq!(result.counts.removed, 0);
    }

    #[test]
    fn test_merge_new_spec_modified_not_allowed() {
        let plan = DeltaPlan {
            added: vec![],
            modified: vec![RequirementBlock {
                header_line: "### Requirement: Something".to_string(),
                name: "Something".to_string(),
                raw: "### Requirement: Something\nModified.".to_string(),
            }],
            removed: vec![],
            renamed: vec![],
            section_presence: SectionPresence {
                added: false,
                modified: true,
                removed: false,
                renamed: false,
            },
        };

        let result = merge_delta_plan("", &plan, "new-spec", "test-change");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("only ADDED requirements are allowed for new specs"));
    }

    #[test]
    fn test_merge_new_spec_renamed_not_allowed() {
        let plan = DeltaPlan {
            added: vec![],
            modified: vec![],
            removed: vec![],
            renamed: vec![RenamePair {
                from: "Old".to_string(),
                to: "New".to_string(),
            }],
            section_presence: SectionPresence {
                added: false,
                modified: false,
                removed: false,
                renamed: true,
            },
        };

        let result = merge_delta_plan("", &plan, "new-spec", "test-change");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("only ADDED requirements are allowed for new specs"));
    }

    #[test]
    fn test_merge_duplicate_added_error() {
        let target = r#"# test-spec Specification

## Purpose
Test.

## Requirements
"#;
        let plan = DeltaPlan {
            added: vec![
                RequirementBlock {
                    header_line: "### Requirement: Duplicate".to_string(),
                    name: "Duplicate".to_string(),
                    raw: "### Requirement: Duplicate\nFirst.".to_string(),
                },
                RequirementBlock {
                    header_line: "### Requirement: Duplicate".to_string(),
                    name: "Duplicate".to_string(),
                    raw: "### Requirement: Duplicate\nSecond.".to_string(),
                },
            ],
            modified: vec![],
            removed: vec![],
            renamed: vec![],
            section_presence: SectionPresence {
                added: true,
                modified: false,
                removed: false,
                renamed: false,
            },
        };

        let result = merge_delta_plan(target, &plan, "test-spec", "test-change");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("duplicate requirement in ADDED"));
    }

    #[test]
    fn test_merge_modified_not_found_error() {
        let target = r#"# test-spec Specification

## Purpose
Test.

## Requirements
### Requirement: Existing
Some content.
"#;
        let plan = DeltaPlan {
            added: vec![],
            modified: vec![RequirementBlock {
                header_line: "### Requirement: Nonexistent".to_string(),
                name: "Nonexistent".to_string(),
                raw: "### Requirement: Nonexistent\nModified.".to_string(),
            }],
            removed: vec![],
            renamed: vec![],
            section_presence: SectionPresence {
                added: false,
                modified: true,
                removed: false,
                renamed: false,
            },
        };

        let result = merge_delta_plan(target, &plan, "test-spec", "test-change");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("MODIFIED failed"));
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_merge_removed_not_found_error() {
        let target = r#"# test-spec Specification

## Purpose
Test.

## Requirements
### Requirement: Existing
Some content.
"#;
        let plan = DeltaPlan {
            added: vec![],
            modified: vec![],
            removed: vec!["Nonexistent".to_string()],
            renamed: vec![],
            section_presence: SectionPresence {
                added: false,
                modified: false,
                removed: true,
                renamed: false,
            },
        };

        let result = merge_delta_plan(target, &plan, "test-spec", "test-change");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("REMOVED failed"));
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_merge_renamed_source_not_found_error() {
        let target = r#"# test-spec Specification

## Purpose
Test.

## Requirements
### Requirement: Existing
Some content.
"#;
        let plan = DeltaPlan {
            added: vec![],
            modified: vec![],
            removed: vec![],
            renamed: vec![RenamePair {
                from: "Nonexistent".to_string(),
                to: "New Name".to_string(),
            }],
            section_presence: SectionPresence {
                added: false,
                modified: false,
                removed: false,
                renamed: true,
            },
        };

        let result = merge_delta_plan(target, &plan, "test-spec", "test-change");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("RENAMED failed"));
        assert!(err.contains("source not found"));
    }

    #[test]
    fn test_merge_renamed_target_exists_error() {
        let target = r#"# test-spec Specification

## Purpose
Test.

## Requirements
### Requirement: First
First content.

### Requirement: Second
Second content.
"#;
        let plan = DeltaPlan {
            added: vec![],
            modified: vec![],
            removed: vec![],
            renamed: vec![RenamePair {
                from: "First".to_string(),
                to: "Second".to_string(),
            }],
            section_presence: SectionPresence {
                added: false,
                modified: false,
                removed: false,
                renamed: true,
            },
        };

        let result = merge_delta_plan(target, &plan, "test-spec", "test-change");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("RENAMED failed"));
        assert!(err.contains("target already exists"));
    }

    #[test]
    fn test_merge_added_already_exists_error() {
        let target = r#"# test-spec Specification

## Purpose
Test.

## Requirements
### Requirement: Existing
Some content.
"#;
        let plan = DeltaPlan {
            added: vec![RequirementBlock {
                header_line: "### Requirement: Existing".to_string(),
                name: "Existing".to_string(),
                raw: "### Requirement: Existing\nNew content.".to_string(),
            }],
            modified: vec![],
            removed: vec![],
            renamed: vec![],
            section_presence: SectionPresence {
                added: true,
                modified: false,
                removed: false,
                renamed: false,
            },
        };

        let result = merge_delta_plan(target, &plan, "test-spec", "test-change");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("ADDED failed"));
        assert!(err.contains("already exists"));
    }

    #[test]
    fn test_merge_no_operations_error() {
        let target = r#"# test-spec Specification

## Purpose
Test.

## Requirements
"#;
        let plan = DeltaPlan {
            added: vec![],
            modified: vec![],
            removed: vec![],
            renamed: vec![],
            section_presence: SectionPresence {
                added: false,
                modified: false,
                removed: false,
                renamed: false,
            },
        };

        let result = merge_delta_plan(target, &plan, "test-spec", "test-change");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no operations"));
    }

    #[test]
    fn test_merge_combined_operations() {
        let target = r#"# test-spec Specification

## Purpose
Test.

## Requirements
### Requirement: Keep
Keep this.

### Requirement: Modify
Old content.

### Requirement: Remove
Remove this.

### Requirement: Rename
Rename this.
"#;
        let plan = DeltaPlan {
            added: vec![RequirementBlock {
                header_line: "### Requirement: Added".to_string(),
                name: "Added".to_string(),
                raw: "### Requirement: Added\nNew requirement.".to_string(),
            }],
            modified: vec![RequirementBlock {
                header_line: "### Requirement: Modify".to_string(),
                name: "Modify".to_string(),
                raw: "### Requirement: Modify\nUpdated content.".to_string(),
            }],
            removed: vec!["Remove".to_string()],
            renamed: vec![RenamePair {
                from: "Rename".to_string(),
                to: "Renamed".to_string(),
            }],
            section_presence: SectionPresence {
                added: true,
                modified: true,
                removed: true,
                renamed: true,
            },
        };

        let result = merge_delta_plan(target, &plan, "test-spec", "test-change").unwrap();
        assert_eq!(result.counts.added, 1);
        assert_eq!(result.counts.modified, 1);
        assert_eq!(result.counts.removed, 1);
        assert_eq!(result.counts.renamed, 1);

        assert!(result.rebuilt.contains("### Requirement: Keep"));
        assert!(result.rebuilt.contains("### Requirement: Modify"));
        assert!(result.rebuilt.contains("Updated content."));
        assert!(!result.rebuilt.contains("### Requirement: Remove"));
        assert!(result.rebuilt.contains("### Requirement: Renamed"));
        assert!(!result.rebuilt.contains("### Requirement: Rename\n"));
        assert!(result.rebuilt.contains("### Requirement: Added"));
    }

    #[test]
    fn test_build_spec_skeleton() {
        let skeleton = build_spec_skeleton("test-capability", "my-change");
        assert!(skeleton.contains("# test-capability Specification"));
        assert!(skeleton.contains("## Purpose"));
        assert!(skeleton.contains("created by archiving change my-change"));
        assert!(skeleton.contains("## Requirements"));
    }

    #[test]
    fn test_extract_requirements_section() {
        let content = r#"# Test Spec

## Purpose
Some purpose.

## Requirements
Some preamble text.

### Requirement: First
First content.

### Requirement: Second
Second content.

## Notes
Some notes.
"#;
        let parts = extract_requirements_section(content);
        assert!(parts.before.contains("## Purpose"));
        assert_eq!(parts.header_line, "## Requirements");
        assert!(parts.preamble.contains("Some preamble text"));
        assert_eq!(parts.body_blocks.len(), 2);
        assert!(parts.after.contains("## Notes"));
    }

    #[test]
    fn test_extract_requirements_section_no_requirements() {
        let content = r#"# Test Spec

## Purpose
Some purpose.
"#;
        let parts = extract_requirements_section(content);
        assert!(parts.before.contains("## Purpose"));
        assert_eq!(parts.header_line, "## Requirements");
        assert!(parts.preamble.is_empty());
        assert!(parts.body_blocks.is_empty());
    }

    #[test]
    fn test_glob_has_matches_simple() {
        let temp_dir = tempfile::tempdir().unwrap();
        let base = temp_dir.path();

        std::fs::create_dir_all(base.join("specs")).unwrap();
        std::fs::write(base.join("specs").join("test.md"), "# Test").unwrap();

        assert!(glob_has_matches(base, "specs/*.md"));
        assert!(!glob_has_matches(base, "docs/*.md"));
    }

    #[test]
    fn test_glob_has_matches_recursive() {
        let temp_dir = tempfile::tempdir().unwrap();
        let base = temp_dir.path();

        std::fs::create_dir_all(base.join("specs").join("subdir")).unwrap();
        std::fs::write(base.join("specs").join("subdir").join("test.md"), "# Test").unwrap();

        assert!(glob_has_matches(base, "specs/**/*.md"));
    }

    #[test]
    fn test_glob_has_matches_no_matches() {
        let temp_dir = tempfile::tempdir().unwrap();
        let base = temp_dir.path();

        std::fs::create_dir_all(base.join("specs")).unwrap();

        assert!(!glob_has_matches(base, "specs/*.md"));
    }

    #[test]
    fn test_artifact_output_exists_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let change_dir = temp_dir.path();

        std::fs::create_dir_all(change_dir.join("output")).unwrap();
        std::fs::write(change_dir.join("output").join("result.txt"), "content").unwrap();

        assert!(artifact_output_exists(change_dir, "output/result.txt"));
        assert!(!artifact_output_exists(change_dir, "output/missing.txt"));
    }

    #[test]
    fn test_artifact_output_exists_glob() {
        let temp_dir = tempfile::tempdir().unwrap();
        let change_dir = temp_dir.path();

        std::fs::create_dir_all(change_dir.join("specs")).unwrap();
        std::fs::write(change_dir.join("specs").join("spec.md"), "# Spec").unwrap();

        assert!(artifact_output_exists(change_dir, "specs/*.md"));
        assert!(!artifact_output_exists(change_dir, "docs/*.md"));
    }

    #[test]
    fn test_find_specs_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        let specs_dir = temp_dir.path().join("specs");

        let specs = find_specs(&specs_dir);
        assert!(specs.is_empty());
    }

    #[test]
    fn test_find_specs_with_specs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let specs_dir = temp_dir.path().join("specs");

        std::fs::create_dir_all(specs_dir.join("capability-a")).unwrap();
        std::fs::write(
            specs_dir.join("capability-a").join("spec.md"),
            "# Capability A\n\n## Purpose\nTest.",
        )
        .unwrap();

        std::fs::create_dir_all(specs_dir.join("capability-b")).unwrap();
        std::fs::write(
            specs_dir.join("capability-b").join("spec.md"),
            "# Capability B\n\n## Purpose\nTest.",
        )
        .unwrap();

        std::fs::create_dir_all(specs_dir.join("empty-capability")).unwrap();

        let specs = find_specs(&specs_dir);
        assert_eq!(specs.len(), 3);

        assert!(specs.iter().any(|s| s.name == "capability-a" && s.exists));
        assert!(specs.iter().any(|s| s.name == "capability-b" && s.exists));
        assert!(specs
            .iter()
            .any(|s| s.name == "empty-capability" && !s.exists));
    }

    #[test]
    fn test_find_spec_updates() {
        let temp_dir = tempfile::tempdir().unwrap();
        let change_dir = temp_dir
            .path()
            .join("openspec")
            .join("changes")
            .join("test-change");
        let main_specs_dir = temp_dir.path().join("openspec").join("specs");

        std::fs::create_dir_all(change_dir.join("specs").join("new-capability")).unwrap();
        std::fs::write(
            change_dir
                .join("specs")
                .join("new-capability")
                .join("spec.md"),
            "# New Capability\n\n## ADDED Requirements\n",
        )
        .unwrap();

        std::fs::create_dir_all(change_dir.join("specs").join("existing-capability")).unwrap();
        std::fs::write(
            change_dir
                .join("specs")
                .join("existing-capability")
                .join("spec.md"),
            "# Existing Capability\n\n## MODIFIED Requirements\n",
        )
        .unwrap();

        std::fs::create_dir_all(main_specs_dir.join("existing-capability")).unwrap();
        std::fs::write(
            main_specs_dir.join("existing-capability").join("spec.md"),
            "# Existing Capability\n\n## Purpose\nTest.",
        )
        .unwrap();

        let updates = find_spec_updates(&change_dir, &main_specs_dir);
        assert_eq!(updates.len(), 2);

        let new_update = updates
            .iter()
            .find(|u| u.spec_name == "new-capability")
            .unwrap();
        assert!(!new_update.target_exists);

        let existing_update = updates
            .iter()
            .find(|u| u.spec_name == "existing-capability")
            .unwrap();
        assert!(existing_update.target_exists);
    }

    #[test]
    fn test_spec_info_sorting() {
        let temp_dir = tempfile::tempdir().unwrap();
        let specs_dir = temp_dir.path().join("specs");

        std::fs::create_dir_all(specs_dir.join("zebra")).unwrap();
        std::fs::write(specs_dir.join("zebra").join("spec.md"), "# Zebra").unwrap();

        std::fs::create_dir_all(specs_dir.join("alpha")).unwrap();
        std::fs::write(specs_dir.join("alpha").join("spec.md"), "# Alpha").unwrap();

        std::fs::create_dir_all(specs_dir.join("middle")).unwrap();
        std::fs::write(specs_dir.join("middle").join("spec.md"), "# Middle").unwrap();

        let specs = find_specs(&specs_dir);
        assert_eq!(specs.len(), 3);
        assert_eq!(specs[0].name, "alpha");
        assert_eq!(specs[1].name, "middle");
        assert_eq!(specs[2].name, "zebra");
    }
}
