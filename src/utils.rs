pub fn is_in_rect<T: PartialOrd>(
    position: (T, T), 
    rect: (T, T, T, T), 
    inclusive_stop: bool
) -> bool {
    let (x, y) = position;
    let (x0, y0, x1, y1) = rect;
    
    if x < x0 {return false;}
    if y < y0 {return false;}
    if inclusive_stop {
        if x > x1 {return false;}
        if y > y1 {return false;}
    }
    else {
        if x >= x1 {return false;}
        if y >= y1 {return false;}
    }
    
    return true;
}