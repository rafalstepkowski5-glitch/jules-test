pub fn svg_line_chart(values: &[f64], color: &str) -> String {
    if values.is_empty(){ return r##"<svg viewBox="0 0 300 80" width="100%" height="80"><text x="10" y="40" fill="#889">brak danych</text></svg>"##.into(); }
    let w=300.0; let h=80.0; let pad=6.0;
    let min=values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max=values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range=if (max-min).abs()<0.001 {1.0} else {max-min};
    let step=if values.len()>1 {(w-2.0*pad)/(values.len()-1) as f64} else {0.0};
    let mut line=String::new(); let mut area=String::new(); let (mut lx, mut ly)=(0.0,0.0);
    for (i,v) in values.iter().enumerate(){
        let x=pad+i as f64*step;
        let y=h-pad-(v-min)/range*(h-2.0*pad);
        lx=x; ly=y;
        if i==0 { line.push_str(&format!("M {x:.2} {y:.2}")); area.push_str(&format!("M {x:.2} {:.2} L {x:.2} {y:.2}", h-pad)); }
        else { line.push_str(&format!(" L {x:.2} {y:.2}")); area.push_str(&format!(" L {x:.2} {y:.2}")); }
    }
    area.push_str(&format!(" L {lx:.2} {:.2} Z", h-pad));
    let id=color.replace('#',"");
    format!(r##"<svg viewBox="0 0 {w} {h}" width="100%" height="80" preserveAspectRatio="none"><defs><linearGradient id="g{id}" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="{color}" stop-opacity="0.35"/><stop offset="100%" stop-color="{color}" stop-opacity="0"/></linearGradient></defs><path d="{area}" fill="url(#g{id})"/><path d="{line}" fill="none" stroke="{color}" stroke-width="2.2" stroke-linecap="round"/><circle cx="{lx:.2}" cy="{ly:.2}" r="3.5" fill="{color}" stroke="#0b1220" stroke-width="1.5"/></svg>"##)
}
