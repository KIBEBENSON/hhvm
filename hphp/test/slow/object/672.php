<?hh

class FOo {
  public function exclAIM() {
 echo "FOo::exclAIM\n";
 }
  public function teST() {
 echo "FOo::teST\n";
 }
  public function __CaLL($name, $args) {
    echo "FOo::" . $name . "\n";
  }
}
class TesT {
  public static function ExclAim() {
    $obj = new fOO();
    $obj->{
__fUNCTION__}
 = 1;
    $obj->{
__cLASS__}
 = 2;
    $obj->__FuNCTION__ = 3;
    $obj->__ClASS__ = 4;
    $obj->{
__FUnCTION__}
();
    $obj->{
__CLaSS__}
();
    $obj->__FUNcTION__();
    $obj->__CLAsS__();
    $arr = array();
    foreach ($obj as $k => $v) {
      $arr[$k] = $v;
    }
    ksort(inout $arr);
    var_dump($arr);
  }
}

<<__EntryPoint>>
function main_672() {
tEst::eXclaiM();
}
