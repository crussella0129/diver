use anyhow::{Context, Result, bail};
use quick_xml::Reader;
use quick_xml::events::Event;

use crate::model::Paper;

#[derive(Debug)]
pub struct FeedResult {
    pub papers: Vec<Paper>,
    pub total_results: u32,
}

pub fn parse_feed(xml: &str) -> Result<FeedResult> {
    let mut reader = Reader::from_str(xml);

    let mut papers = Vec::new();
    let mut total_results: u32 = 0;

    let mut in_entry = false;
    let mut current_tag = String::new();
    let mut depth_tag = String::new();

    let mut title = String::new();
    let mut summary = String::new();
    let mut published = String::new();
    let mut updated = String::new();
    let mut arxiv_id = String::new();
    let mut pdf_url = String::new();
    let mut primary_category = String::new();
    let mut authors: Vec<String> = Vec::new();
    let mut in_author = false;
    let mut author_name = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local = local_name(e.name().as_ref());
                let local_str = String::from_utf8_lossy(&local).to_string();

                if local_str == "entry" {
                    in_entry = true;
                    title.clear();
                    summary.clear();
                    published.clear();
                    updated.clear();
                    arxiv_id.clear();
                    pdf_url.clear();
                    primary_category.clear();
                    authors.clear();
                } else if in_entry {
                    if local_str == "author" {
                        in_author = true;
                        author_name.clear();
                    } else if local_str == "link" {
                        let mut href = String::new();
                        let mut link_title = String::new();
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            let val = String::from_utf8_lossy(&attr.value).to_string();
                            if key == "href" {
                                href = val;
                            } else if key == "title" {
                                link_title = val;
                            }
                        }
                        if link_title == "pdf" {
                            pdf_url = href;
                        } else if arxiv_id.is_empty() && href.contains("arxiv.org/abs/") {
                            arxiv_id = href.rsplit("abs/").next().unwrap_or("").to_string();
                        }
                    } else if (local_str == "primary_category" || local_str == "category")
                        && primary_category.is_empty()
                    {
                        for attr in e.attributes().flatten() {
                            let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                            if key == "term" {
                                primary_category = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                    current_tag = local_str;
                } else {
                    if local_str == "totalResults" {
                        depth_tag = "totalResults".to_string();
                    }
                    current_tag = local_str;
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().context("invalid XML text")?.trim().to_string();
                if !text.is_empty() {
                    if in_entry {
                        match current_tag.as_str() {
                            "title" => title = text,
                            "summary" => summary = text,
                            "published" => published = text,
                            "updated" => updated = text,
                            "name" if in_author => author_name = text,
                            "id" if arxiv_id.is_empty() => {
                                arxiv_id = text.rsplit("abs/").next().unwrap_or(&text).to_string();
                            }
                            _ => {}
                        }
                    } else if depth_tag == "totalResults" {
                        total_results = text.parse().unwrap_or(0);
                        depth_tag.clear();
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let local = local_name(e.name().as_ref());
                let local_str = String::from_utf8_lossy(&local).to_string();

                if local_str == "entry" {
                    papers.push(Paper {
                        title: title.clone(),
                        authors: authors.clone(),
                        summary: summary.clone(),
                        primary_category: primary_category.clone(),
                        published: published.clone(),
                        updated: updated.clone(),
                        arxiv_id: arxiv_id.clone(),
                        pdf_url: pdf_url.clone(),
                    });
                    in_entry = false;
                } else if local_str == "author" && in_author {
                    if !author_name.is_empty() {
                        authors.push(author_name.clone());
                    }
                    in_author = false;
                } else if local_str == "totalResults" {
                    depth_tag.clear();
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => bail!(
                "XML parse error at position {}: {e}",
                reader.error_position()
            ),
            _ => {}
        }
    }

    Ok(FeedResult {
        papers,
        total_results,
    })
}

fn local_name(name: &[u8]) -> Vec<u8> {
    if let Some(pos) = name.iter().position(|&b| b == b':') {
        name[pos + 1..].to_vec()
    } else {
        name.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_FEED: &str = include_str!("../tests/fixtures/sample_feed.xml");
    const EMPTY_FEED: &str = include_str!("../tests/fixtures/empty_feed.xml");

    #[test]
    fn test_parse_valid_feed() {
        let result = parse_feed(SAMPLE_FEED).unwrap();
        assert_eq!(result.papers.len(), 3);
        for paper in &result.papers {
            assert!(!paper.title.is_empty());
            assert!(!paper.authors.is_empty());
        }
    }

    #[test]
    fn test_parse_entry_fields() {
        let result = parse_feed(SAMPLE_FEED).unwrap();
        let paper = &result.papers[0];
        assert!(!paper.title.is_empty());
        assert!(!paper.authors.is_empty());
        assert!(!paper.summary.is_empty());
        assert!(!paper.primary_category.is_empty());
        assert!(!paper.arxiv_id.is_empty());
    }

    #[test]
    fn test_parse_total_results() {
        let result = parse_feed(SAMPLE_FEED).unwrap();
        assert_eq!(result.total_results, 12345);
    }

    #[test]
    fn test_parse_empty_feed() {
        let result = parse_feed(EMPTY_FEED).unwrap();
        assert!(result.papers.is_empty());
        assert_eq!(result.total_results, 0);
    }

    #[test]
    fn test_parse_malformed_xml() {
        let result = parse_feed("<broken><xml");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("XML parse error"), "got: {err_msg}");
    }
}
