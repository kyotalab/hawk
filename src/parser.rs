use crate::Error;

pub fn parse_query_segments(query: &str) -> Result<(&str, &str), Error> {
    let mut segments = query.split('.');
    segments.next();
    let segment = segments.next().ok_or(Error::InvalidQuery("Missing field segment in query".into()))?; 
    let param = segments.next().ok_or(Error::InvalidQuery("Missing parameter segment in query".into()))?;

    Ok((segment, param))
}

pub fn parse_array_segment(segment: &str) -> Result<(usize, usize), Error> {
    let idx = segment.find('[').ok_or(Error::InvalidQuery("Missing '[' in segment".into()))?;
    let ridx = segment.find(']').ok_or(Error::InvalidQuery("Missing ']' in segment".into()))?;

    if idx >= ridx {
        return Err(Error::InvalidQuery("Invalid bracket order".into()));
    }

    Ok((idx, ridx))
}
